import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  const wasmLogs = [];

  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('[WASM]')) {
      wasmLogs.push(text);
    }
  });

  console.log('Test: WASM should only initialize ONCE across page navigations\n');

  console.log('Step 1: Load home page (triggers WASM init)...');
  await page.goto('http://localhost:9867/', { waitUntil: 'networkidle' });
  await page.waitForTimeout(2000);

  console.log('Step 2: Navigate to /config page...');
  await page.click('text=Config');
  await page.waitForTimeout(3000);

  console.log('Step 3: Navigate to /simulator page...');
  await page.click('text=Simulator');
  await page.waitForTimeout(3000);

  console.log('Step 4: Navigate back to /config...');
  await page.click('text=Config');
  await page.waitForTimeout(3000);

  console.log('\n=== WASM INITIALIZATION LOGS ===');
  wasmLogs.forEach(log => console.log(log));

  const initCount = wasmLogs.filter(log => log.includes('Starting global initialization')).length;
  const completeCount = wasmLogs.filter(log => log.includes('initialization complete')).length;

  console.log('\n=== RESULTS ===');
  console.log(`WASM initialization started: ${initCount} time(s)`);
  console.log(`WASM initialization completed: ${completeCount} time(s)`);

  if (initCount === 1 && completeCount === 1) {
    console.log('\n✅ SUCCESS: WASM cached properly! Initialized only once.');
  } else {
    console.log('\n❌ FAIL: WASM initialized multiple times (not cached)');
  }

  await browser.close();
  process.exit(initCount === 1 ? 0 : 1);
})();
