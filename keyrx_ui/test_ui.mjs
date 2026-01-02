import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  const errors = [];
  const consoleMessages = [];

  // Capture console messages
  page.on('console', msg => {
    const text = msg.text();
    consoleMessages.push({ type: msg.type(), text });
    if (msg.type() === 'error') {
      errors.push(text);
    }
  });

  // Capture page errors
  page.on('pageerror', error => {
    errors.push(`PAGE ERROR: ${error.toString()}\n${error.stack}`);
  });

  try {
    console.log('Loading http://localhost:9867...');
    await page.goto('http://localhost:9867', {
      waitUntil: 'networkidle',
      timeout: 10000
    });

    // Wait a bit for React to initialize
    await page.waitForTimeout(2000);

    // Take screenshot
    await page.screenshot({ path: '/tmp/ui_screenshot.png' });
    console.log('Screenshot saved to /tmp/ui_screenshot.png');

    // Check if root div has content
    const rootContent = await page.evaluate(() => {
      const root = document.getElementById('root');
      return {
        hasContent: root && root.children.length > 0,
        innerHTML: root ? root.innerHTML.substring(0, 500) : 'NO ROOT ELEMENT'
      };
    });

    console.log('\n=== PAGE LOAD RESULTS ===');
    console.log('Root element has content:', rootContent.hasContent);
    console.log('Root innerHTML preview:', rootContent.innerHTML);

    console.log('\n=== CONSOLE MESSAGES ===');
    consoleMessages.forEach(msg => {
      console.log(`[${msg.type.toUpperCase()}] ${msg.text}`);
    });

    if (errors.length > 0) {
      console.log('\n=== ERRORS DETECTED ===');
      errors.forEach((err, i) => {
        console.log(`\nError ${i + 1}:`);
        console.log(err);
      });
      process.exit(1);
    } else {
      console.log('\n=== NO ERRORS DETECTED ===');
      console.log('Page loaded successfully!');
      process.exit(0);
    }

  } catch (error) {
    console.error('\n=== LOAD FAILED ===');
    console.error(error.message);
    await page.screenshot({ path: '/tmp/ui_error_screenshot.png' });
    process.exit(1);
  } finally {
    await browser.close();
  }
})();
