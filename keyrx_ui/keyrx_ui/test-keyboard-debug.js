const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  await page.goto('http://localhost:5173/config');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);

  // Take screenshot
  await page.screenshot({ path: '/tmp/config-page.png', fullPage: true });
  console.log('Screenshot saved to /tmp/config-page.png');

  // Check for keyboard-visualizer
  const visualizer = page.locator('[data-testid="keyboard-visualizer"]');
  const count = await visualizer.count();
  console.log('Keyboard visualizer elements:', count);

  if (count > 0) {
    const isVisible = await visualizer.first().isVisible();
    console.log('First visualizer visible:', isVisible);

    const box = await visualizer.first().boundingBox();
    console.log('Bounding box:', box);

    // Check SVG
    const svg = visualizer.first().locator('svg');
    const svgCount = await svg.count();
    console.log('SVG elements inside:', svgCount);

    if (svgCount > 0) {
      const svgBox = await svg.first().boundingBox();
      console.log('SVG bounding box:', svgBox);

      // Check for key groups
      const keyGroups = await svg.first().locator('g.key-group').count();
      console.log('Key groups found:', keyGroups);
    }
  }

  await browser.close();
})();
