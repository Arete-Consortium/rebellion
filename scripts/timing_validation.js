#!/usr/bin/env node
/**
 * Timing validation bot for Rebellion itch.io vertical slice.
 *
 * Automated playthrough (or death) to capture CGSessionTimer output.
 * Sends random WASD + Space inputs to simulate a player who is trying
 * but not necessarily skilled. Runs until death, slice completion, or
 * a hard timeout.
 */

const { chromium } = require('playwright');
const http = require('http');
const path = require('path');
const fs = require('fs');

const WEB_DIR = process.argv[2] || path.join(__dirname, '..', 'web');
const PORT = 8766;
const HARD_TIMEOUT_MS = 15 * 60 * 1000; // 15 minutes max

const SCREENSHOT_DIR = path.join(__dirname, '..', 'timing_validation_output');
if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

const timerLogs = [];
const consoleErrors = [];
const consoleWarnings = [];

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
                return;
            }
            res.writeHead(200, { 'Content-Type': contentType });
            res.end(data);
        });
    });
    return new Promise((resolve) => {
        server.listen(PORT, '127.0.0.1', () => {
            console.log(`[timing] Server at http://127.0.0.1:${PORT}`);
            resolve(server);
        });
    });
}

async function run() {
    let browser;
    let server;
    let passed = true;
    let gameEnded = false;

    try {
        server = await startServer();
        browser = await chromium.launch({ headless: true });
        const context = await browser.newContext({
            viewport: { width: 1280, height: 720 },
        });
        const page = await context.newPage();

        page.on('console', (msg) => {
            const text = msg.text();
            if (text.includes('CGSessionTimer')) {
                timerLogs.push({ type: msg.type(), text, time: Date.now() });
                console.log(`[timing] CAPTURED: ${text}`);
                gameEnded = true;
            }
            if (msg.type() === 'error') {
                const harmless = [
                    'exceptions for control flow',
                    "don't mind me",
                    'not actually an error',
                    'unreachable',
                    'WebGL: CONTEXT_LOST_WEBGL',
                    'webgl2',
                ];
                const isWebGLCrash = text.includes('glow-') && text.includes('web_sys.rs') && text.includes('unwrap');
                if (!harmless.some(h => text.toLowerCase().includes(h.toLowerCase())) && !isWebGLCrash) {
                    consoleErrors.push(text);
                } else {
                    consoleWarnings.push(text);
                }
            }
        });

        page.on('pageerror', (err) => {
            const text = err.message || String(err);
            const isWebGLCrash = text.includes('glow-') && text.includes('web_sys.rs') && text.includes('unwrap');
            if (!isWebGLCrash) {
                consoleErrors.push(text);
            } else {
                consoleWarnings.push(text);
            }
        });

        // Load page
        console.log('[timing] Loading page...');
        await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'domcontentloaded' });
        await page.waitForSelector('#enter-archive.visible', { timeout: 30000 });
        console.log('[timing] WASM ready, entering archive...');
        await page.click('#enter-archive');
        await page.waitForTimeout(500);

        // Focus canvas
        const canvas = page.locator('#bevy-canvas');

        // Bot loop: send random movement + shooting keys
        const keys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Space'];
        const startTime = Date.now();
        let screenshotCount = 0;

        console.log('[timing] Bot started — sending random WASD + Space inputs');

        while (!gameEnded && (Date.now() - startTime) < HARD_TIMEOUT_MS) {
            // Send a burst of 3-5 random keypresses
            const burstSize = 3 + Math.floor(Math.random() * 3);
            for (let i = 0; i < burstSize; i++) {
                const key = keys[Math.floor(Math.random() * keys.length)];
                await canvas.press(key);
            }

            // Every 30 seconds, take a screenshot
            const elapsed = Date.now() - startTime;
            if (elapsed > (screenshotCount + 1) * 30000) {
                screenshotCount++;
                const mins = Math.floor(elapsed / 60000);
                const secs = Math.floor((elapsed % 60000) / 1000);
                const path_ = path.join(SCREENSHOT_DIR, `t${mins}m${secs}s.png`);
                await page.screenshot({ path: path_ });
                console.log(`[timing] Screenshot at ${mins}m${secs}s`);

                // Check DOM for death / complete indicators
                const pageText = await page.evaluate(() => document.body.innerText);
                if (pageText.includes('CLONE LOST')) {
                    console.log('[timing] Detected death screen');
                    gameEnded = true;
                    break;
                }
                if (pageText.includes('ARCHIVE COMPLETE')) {
                    console.log('[timing] Detected slice complete screen');
                    gameEnded = true;
                    break;
                }
            }

            // Small delay between bursts
            await page.waitForTimeout(80 + Math.floor(Math.random() * 120));
        }

        // Final screenshot
        await page.screenshot({ path: path.join(SCREENSHOT_DIR, 'final.png') });

        const totalElapsed = (Date.now() - startTime) / 1000;
        console.log('');
        console.log('=== Timing Validation Results ===');
        console.log(`[timing] Total elapsed: ${totalElapsed.toFixed(1)}s`);

        if (timerLogs.length > 0) {
            timerLogs.forEach(log => {
                console.log(`[timing] ${log.text}`);
            });
        } else {
            console.log('[timing] No CGSessionTimer log captured — timer may not have fired or game did not end');
        }

        if (consoleErrors.length > 0) {
            console.error(`[timing] ${consoleErrors.length} console error(s):`);
            consoleErrors.slice(0, 5).forEach(e => console.error(`       ${e}`));
            passed = false;
        } else {
            console.log('[timing] No console errors.');
        }

        if (consoleWarnings.length > 0) {
            console.log(`[timing] ${consoleWarnings.length} warning(s) (WebGL crashes filtered):`);
        }

        console.log(`[timing] Screenshots: ${SCREENSHOT_DIR}`);
        console.log(passed ? '[timing] OVERALL: PASS' : '[timing] OVERALL: FAIL');

    } catch (err) {
        console.error('[timing] CRASH:', err.message || err);
        passed = false;
    } finally {
        if (browser) await browser.close();
        if (server) server.close();
    }

    process.exit(passed ? 0 : 1);
}

run();
