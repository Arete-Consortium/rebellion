#!/usr/bin/env node
/**
 * Timing validation bot — SLOW VERSION.
 *
 * Sends one keypress every 400-800ms to avoid overwhelming the input system.
 * Monitors console for CGSessionTimer output.
 */

const { chromium } = require('playwright');
const http = require('http');
const path = require('path');
const fs = require('fs');

const WEB_DIR = process.argv[2] || path.join(__dirname, '..', 'web');
const PORT = 8768;
const HARD_TIMEOUT_MS = 18 * 60 * 1000;

const timerLogs = [];
const consoleErrors = [];

async function startServer() {
    const server = http.createServer((req, res) => {
        const filePath = path.join(WEB_DIR, req.url === '/' ? 'index.html' : req.url);
        const ext = path.extname(filePath);
        const mimeTypes = {
            '.html': 'text/html', '.js': 'application/javascript',
            '.wasm': 'application/wasm', '.json': 'application/json',
            '.png': 'image/png', '.jpg': 'image/jpeg', '.css': 'text/css',
        };
        fs.readFile(filePath, (err, data) => {
            if (err) { res.writeHead(404); res.end(); return; }
            res.writeHead(200, { 'Content-Type': mimeTypes[ext] || 'application/octet-stream' });
            res.end(data);
        });
    });
    return new Promise((resolve) => server.listen(PORT, '127.0.0.1', () => resolve(server)));
}

async function run() {
    let browser, server;
    let gameEnded = false;

    try {
        server = await startServer();
        browser = await chromium.launch({ headless: true });
        const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });

        page.on('console', (msg) => {
            const text = msg.text();
            if (text.includes('CGSessionTimer')) {
                timerLogs.push(text);
                console.log(`[timing] CAPTURED: ${text}`);
                gameEnded = true;
            }
            if (msg.type() === 'error') {
                const harmless = [
                    'exceptions for control flow', "don't mind me",
                    'not actually an error', 'unreachable',
                    'WebGL: CONTEXT_LOST_WEBGL', 'webgl2',
                ];
                const isWebGLCrash = text.includes('glow-') && text.includes('web_sys.rs') && text.includes('unwrap');
                if (!harmless.some(h => text.toLowerCase().includes(h.toLowerCase())) && !isWebGLCrash) {
                    consoleErrors.push(text);
                }
            }
        });

        page.on('pageerror', (err) => {
            const text = err.message || String(err);
            const isWebGLCrash = text.includes('glow-') && text.includes('web_sys.rs') && text.includes('unwrap');
            if (!isWebGLCrash) consoleErrors.push(text);
        });

        await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'domcontentloaded' });
        await page.waitForSelector('#enter-archive.visible', { timeout: 30000 });
        await page.click('#enter-archive');
        await page.waitForTimeout(500);

        const canvas = page.locator('#bevy-canvas');
        const keys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Space'];
        const startTime = Date.now();
        let cycle = 0;

        console.log('[timing] Slow bot started — one key every ~600ms');

        while (!gameEnded && (Date.now() - startTime) < HARD_TIMEOUT_MS) {
            const key = keys[cycle % keys.length];
            await canvas.press(key);
            cycle++;

            // Every ~30s of wall-clock, log progress
            const elapsed = Date.now() - startTime;
            if (elapsed % 30000 < 700) {
                const mins = Math.floor(elapsed / 60000);
                const secs = Math.floor((elapsed % 60000) / 1000);
                console.log(`[timing] ${mins}m ${secs}s — keys sent: ${cycle}, errors: ${consoleErrors.length}`);
            }

            await page.waitForTimeout(400 + Math.floor(Math.random() * 400));
        }

        const total = (Date.now() - startTime) / 1000;
        console.log('\n=== Timing Validation Results ===');
        console.log(`[timing] Total elapsed: ${total.toFixed(1)}s`);
        if (timerLogs.length) {
            timerLogs.forEach(l => console.log(`[timing] ${l}`));
        } else {
            console.log('[timing] No CGSessionTimer log captured.');
        }
        if (consoleErrors.length) {
            console.error(`[timing] ${consoleErrors.length} console error(s):`);
            consoleErrors.slice(0, 5).forEach(e => console.error(`       ${e}`));
        } else {
            console.log('[timing] No console errors.');
        }

    } catch (err) {
        console.error('[timing] CRASH:', err.message || err);
    } finally {
        if (browser) await browser.close();
        if (server) server.close();
    }
}

run();
