import { getAllTestCases } from './test-cases/api-tests.js';
const allTests = getAllTestCases();
console.log('Total test cases:', allTests.length);
const deviceTests = allTests.filter(t => t.id.startsWith('devices-004'));
console.log('\nDevice rename tests:', deviceTests.length);
deviceTests.forEach(t => console.log('  -', t.id, ':', t.name));
