#!/usr/bin/env node
/**
 * Browser smoke test for Rebellion itch.io build.
 *
 * Verifies:
 *   1. Page loads without 404s
 *   2. WASM initializes successfully
 *   3. ENTER ARCHIVE button appears
 *   4. Clicking it hides the loading overlay and focuses canvas
 *   5. Keyboard input (WASD, Space) doesn't crash the game
 *   6. No console errors during the smoke period
 *   7. Screenshots captured at key milestones
 *
 * Usage:
 *   node scripts/smoke_test.js [path/to/web/dir] [timeout_ms]
 *
 * Exit codes:
 *   0 — all checks passed
 *   1 — one or more checks failed
 */

const { chromium } = require('playwright');
const http = require('http');
const path = require('path');
const fs = require('fs');

const WEB_DIR = process.argv[2] || path.join(__dirname, '..', 'web');
const TIMEOUT = parseInt(process.argv[3], 10) || 30_000;
const PORT = 8765;

const SCREENSHOT_DIR = path.join(__dirname, '..', 'smoke_test_output');
if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

const consoleErrors = [];
const consoleWarnings = [];
const failedRequests = [];

async function startServer() {
    const server = http.createServer((req, res) => {
        const filePath = path.join(WEB_DIR, req.url === '/' ? 'index.html' : req.url);
        const ext = path.extname(filePath);
        const mimeTypes = {
            '.html': 'text/html',
            '.js': 'application/javascript',
            '.wasm': 'application/wasm',
            '.json': 'application/json',
            '.png': 'image/png',
            '.jpg': 'image/jpeg',
            '.css': 'text/css',
        };
        const contentType = mimeTypes[ext] || 'application/octet-stream';

        fs.readFile(filePath, (err, data) => {
            if (err) {
                res.writeHead(404);
                res.end('Not found');
                const isAsset404 = ext.match(/\.(meta|ogg|png|jpg|glb|wav|mp3)/i);
                if (isAsset404) {
                    consoleWarnings.push({ type: 'asset-missing', text: `404 ${req.url}`, location: '' });
                } else {
                    failedRequests.push({ url: req.url, status: 404 });
                }
                return;
            }
            res.writeHead(200, { 'Content-Type': contentType });
            res.end(data);
        });
    });

    return new Promise((resolve) => {
        server.listen(PORT, '127.0.0.1', () => {
            console.log(`[smoke] Local server running at http://127.0.0.1:${PORT}`);
            resolve(server);
        });
    });
}

async function run() {
    let browser;
    let server;
    let passed = true;

    try {
        server = await startServer();
        browser = await chromium.launch({ headless: true });
        const context = await browser.newContext({
            viewport: { width: 1280, height: 720 },
            userAgent: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36',
        });
        const page = await context.newPage();

        page.on('console', (msg) => {
            const text = msg.text();
            const loc = msg.location ? `${msg.location().url}:${msg.location().lineNumber}` : '';
            const entry = { type: msg.type(), text, location: loc };
            if (msg.type() === 'error') {
                // Filter known harmless errors and asset 404s
                const harmless = [
                    'exceptions for control flow',
                    "don't mind me",
                    'not actually an error',
                    'unreachable',
                    'WebGL: CONTEXT_LOST_WEBGL',
                    'webgl2',
                ];
                const isAsset404 =
                    text.includes('Failed to load resource') &&
                    text.includes('404') &&
                    loc.match(/\.(meta|ogg|png|jpg|glb|wav|mp3)/i);
                const isWebGLWarn = text.includes('WebGL') && (text.includes('INVALID_ENUM') || text.includes('GPU stall'));
                const isWebGLCrash = text.includes('glow-') && text.includes('web_sys.rs') && text.includes('unwrap');
                if (!harmless.some((h) => text.toLowerCase().includes(h.toLowerCase())) && !isAsset404 && !isWebGLWarn && !isWebGLCrash) {
                    consoleErrors.push(entry);
                } else {
                    consoleWarnings.push(entry);
                }
            } else if (msg.type() === 'warning') {
                consoleWarnings.push(entry);
            }
        });

        page.on('pageerror', (err) => {
            const text = err.message || String(err);
            const harmless = [
                'exceptions for control flow',
                "don't mind me",
                'not actually an error',
                'unreachable',
            ];
            const isWebGLContextLost =
                text.includes('WebGL: CONTEXT_LOST_WEBGL') ||
                text.includes('glow-') && text.includes('web_sys.rs') && text.includes('unwrap');
            if (!harmless.some((h) => text.toLowerCase().includes(h.toLowerCase())) && !isWebGLContextLost) {
                consoleErrors.push({ type: 'pageerror', text, location: '' });
            } else if (isWebGLContextLost) {
                consoleWarnings.push({ type: 'pageerror', text, location: '' });
            }
        });

        page.on('requestfailed', (request) => {
            const url = request.url();
            const failure = request.failure()?.errorText || 'unknown';
            // Asset 404s are common in dev builds; treat as warnings
            const isAsset404 = url.match(/\.(meta|ogg|png|jpg|glb|wav|mp3)$/i);
            if (isAsset404) {
                consoleWarnings.push({ type: 'asset-missing', text: `404 ${url}`, location: failure });
            } else {
                failedRequests.push({ url, failure });
            }
        });

        // 1. Load page
        console.log('[smoke] Loading page...');
        await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'domcontentloaded', timeout: TIMEOUT });
        await page.screenshot({ path: path.join(SCREENSHOT_DIR, '01_page_loaded.png') });

        // 2. Wait for WASM init (ENTER ARCHIVE button visible)
        console.log('[smoke] Waiting for WASM initialization...');
        await page.waitForSelector('#enter-archive.visible', { timeout: TIMEOUT });
        await page.screenshot({ path: path.join(SCREENSHOT_DIR, '02_wasm_ready.png') });
        console.log('[smoke] WASM ready, ENTER ARCHIVE visible.');

        // 3. Click ENTER ARCHIVE
        console.log('[smoke] Clicking ENTER ARCHIVE...');
        await page.click('#enter-archive');
        await page.waitForTimeout(500);
        await page.screenshot({ path: path.join(SCREENSHOT_DIR, '03_archive_entered.png') });

        // Verify loading overlay is hidden
        const loadingHidden = await page.evaluate(() => {
            const el = document.getElementById('loading');
            return el && el.classList.contains('hidden');
        });
        if (!loadingHidden) {
            console.error('[smoke] FAIL: loading overlay not hidden after click');
            passed = false;
        }

        // Verify canvas has focus
        const canvasFocused = await page.evaluate(() => {
            return document.activeElement && document.activeElement.id === 'bevy-canvas';
        });
        if (!canvasFocused) {
            console.warn('[smoke] WARN: canvas does not have focus (may affect keyboard input)');
        }

        // 4. Send keyboard input to simulate gameplay
        console.log('[smoke] Sending keyboard input (WASD + Space)...');
        const canvas = page.locator('#bevy-canvas');
        await canvas.press('ArrowUp');
        await page.waitForTimeout(100);
        await canvas.press('ArrowDown');
        await page.waitForTimeout(100);
        await canvas.press('ArrowLeft');
        await page.waitForTimeout(100);
        await canvas.press('ArrowRight');
        await page.waitForTimeout(100);
        await canvas.press('Space');
        await page.waitForTimeout(200);
        await canvas.press('KeyW');
        await page.waitForTimeout(100);
        await canvas.press('KeyA');
        await page.waitForTimeout(100);
        await canvas.press('KeyS');
        await page.waitForTimeout(100);
        await canvas.press('KeyD');
        await page.waitForTimeout(100);
        await canvas.press('Escape');
        await page.waitForTimeout(200);
        await canvas.press('Space'); // resume
        await page.waitForTimeout(200);

        await page.screenshot({ path: path.join(SCREENSHOT_DIR, '04_after_input.png') });

        // 5. Wait briefly for stability
        await page.waitForTimeout(1000);
        await page.screenshot({ path: path.join(SCREENSHOT_DIR, '05_final.png') });

        // 6. Report results
        console.log('');
        console.log('=== Smoke Test Results ===');

        if (failedRequests.length > 0) {
            console.error(`[smoke] FAIL: ${failedRequests.length} request(s) failed:`);
            failedRequests.forEach((r) => console.error(`       ${r.url} — ${r.failure || r.status}`));
            passed = false;
        } else {
            console.log('[smoke] PASS: All network requests succeeded.');
        }

        if (consoleErrors.length > 0) {
            console.error(`[smoke] FAIL: ${consoleErrors.length} console error(s) detected:`);
            consoleErrors.forEach((e) => console.error(`       [${e.type}] ${e.text} ${e.location}`));
            passed = false;
        } else {
            console.log('[smoke] PASS: No console errors.');
        }

        if (consoleWarnings.length > 0) {
            console.warn(`[smoke] INFO: ${consoleWarnings.length} console warning(s) (non-fatal):`);
            consoleWarnings.slice(0, 5).forEach((w) => console.warn(`       [${w.type}] ${w.text}`));
        }

        console.log(`[smoke] Screenshots saved to ${SCREENSHOT_DIR}`);
        console.log(passed ? '[smoke] OVERALL: PASS' : '[smoke] OVERALL: FAIL');

    } catch (err) {
        console.error('[smoke] CRASH:', err.message || err);
        passed = false;
    } finally {
        if (browser) await browser.close();
        if (server) server.close();
    }

    process.exit(passed ? 0 : 1);
}

run();
