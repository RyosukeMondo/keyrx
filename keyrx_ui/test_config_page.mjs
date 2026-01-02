import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  const errors = [];
  const warnings = [];
  const allLogs = [];

  page.on('console', msg => {
    const text = msg.text();
    allLogs.push(`[${msg.type()}] ${text}`);
    if (msg.type() === 'error') {
      errors.push(text);
    } else if (msg.type() === 'warning') {
      warnings.push(text);
    }
  });

  page.on('pageerror', error => {
    errors.push(`PAGE ERROR: ${error.toString()}`);
  });

  console.log('Loading home page first...');
  await page.goto('http://localhost:9867/', { waitUntil: 'networkidle', timeout: 30000 });

  console.log('Clicking Config menu item...');
  await page.click('text=Config');
  await page.waitForTimeout(2000);

  console.log('Current URL:', page.url());

  console.log('Waiting 8 seconds for page to load...');
  await page.waitForTimeout(8000);

  const editorInfo = await page.evaluate(() => {
    const monacoEditor = document.querySelector('.monaco-editor');
    const loadingText = document.body.innerText.includes('Loading');
    const hasCodeEditor = document.body.innerText.includes('Code Editor');
    const hasVisualEditor = document.body.innerText.includes('Visual Editor');
    return {
      hasMonacoEditor: !!monacoEditor,
      hasLoadingText: loadingText,
      hasCodeEditor: hasCodeEditor,
      hasVisualEditor: hasVisualEditor,
      url: window.location.pathname,
      bodyText: document.body.innerText.substring(0, 800)
    };
  });

  console.log('\n=== EDITOR STATUS ===');
  console.log('Current URL:', editorInfo.url);
  console.log('Monaco editor element found:', editorInfo.hasMonacoEditor);
  console.log('Has "Loading" text:', editorInfo.hasLoadingText);
  console.log('Has Code Editor:', editorInfo.hasCodeEditor);
  console.log('Has Visual Editor:', editorInfo.hasVisualEditor);

  console.log('\n=== PAGE CONTENT (first 500 chars) ===');
  console.log(editorInfo.bodyText);

  if (errors.length > 0) {
    console.log('\n=== ERRORS ===');
    errors.forEach((err, i) => console.log(`${i + 1}. ${err}`));
  } else {
    console.log('\n=== NO ERRORS ===');
  }

  if (warnings.length > 0) {
    console.log('\n=== WARNINGS ===');
    warnings.forEach((warn, i) => console.log(`${i + 1}. ${warn}`));
  }

  console.log('\n=== ALL CONSOLE LOGS ===');
  allLogs.forEach(log => console.log(log));

  await browser.close();
  process.exit(errors.length > 0 ? 1 : 0);
})();
