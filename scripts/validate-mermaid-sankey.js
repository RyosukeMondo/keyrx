#!/usr/bin/env node
/**
 * Validates Mermaid Sankey diagrams for circular references
 * Usage: node scripts/validate-mermaid-sankey.js <file.md>
 */

const fs = require('fs');
const path = require('path');

function extractSankeyDiagram(content) {
  const mermaidRegex = /```mermaid\s*\n([\s\S]*?)\n```/g;
  const matches = [];
  let match;

  while ((match = mermaidRegex.exec(content)) !== null) {
    const diagram = match[1];
    if (diagram.includes('sankey')) {
      matches.push(diagram);
    }
  }

  return matches;
}

function parseSankeyEdges(diagram) {
  const edges = [];
  const lines = diagram.split('\n');

  for (const line of lines) {
    const trimmed = line.trim();

    // Skip comments and config lines
    if (trimmed.startsWith('%%') || trimmed.startsWith('sankey') || !trimmed) {
      continue;
    }

    // Parse edge: "Source,Target,Value"
    const parts = trimmed.split(',').map(p => p.trim());

    if (parts.length === 3) {
      // Last part should be the value (number)
      const value = parseInt(parts[2]);

      if (isNaN(value)) {
        console.error(`❌ ERROR: Invalid edge format on line: "${trimmed}"`);
        console.error(`   Expected: "Source,Target,Value" where Value is a number`);
        console.error(`   Got: "${parts[0]},${parts[1]},${parts[2]}"`);
        return null;
      }

      edges.push({ source: parts[0], target: parts[1], value });
    }
  }

  return edges;
}

function detectCycles(edges) {
  const graph = new Map();
  const nodes = new Set();

  // Build adjacency list
  for (const edge of edges) {
    nodes.add(edge.source);
    nodes.add(edge.target);

    if (!graph.has(edge.source)) {
      graph.set(edge.source, []);
    }
    graph.get(edge.source).push(edge.target);
  }

  // DFS to detect cycles
  const visited = new Set();
  const recStack = new Set();
  const cycles = [];

  function dfs(node, path = []) {
    visited.add(node);
    recStack.add(node);
    path.push(node);

    const neighbors = graph.get(node) || [];
    for (const neighbor of neighbors) {
      if (!visited.has(neighbor)) {
        if (dfs(neighbor, [...path])) {
          return true;
        }
      } else if (recStack.has(neighbor)) {
        // Cycle detected
        const cycleStart = path.indexOf(neighbor);
        const cycle = [...path.slice(cycleStart), neighbor];
        cycles.push(cycle);
        return true;
      }
    }

    recStack.delete(node);
    return false;
  }

  for (const node of nodes) {
    if (!visited.has(node)) {
      dfs(node);
    }
  }

  return cycles;
}

function validateFile(filePath) {
  console.log(`\n🔍 Validating: ${filePath}\n`);

  if (!fs.existsSync(filePath)) {
    console.error(`❌ File not found: ${filePath}`);
    process.exit(1);
  }

  const content = fs.readFileSync(filePath, 'utf8');
  const diagrams = extractSankeyDiagram(content);

  if (diagrams.length === 0) {
    console.log('✅ No Sankey diagrams found in file');
    return true;
  }

  console.log(`Found ${diagrams.length} Sankey diagram(s)\n`);

  let hasErrors = false;

  for (let i = 0; i < diagrams.length; i++) {
    console.log(`📊 Diagram ${i + 1}:`);

    const edges = parseSankeyEdges(diagrams[i]);

    if (edges === null) {
      hasErrors = true;
      continue;
    }

    console.log(`   Edges: ${edges.length}`);

    const cycles = detectCycles(edges);

    if (cycles.length > 0) {
      console.error(`\n❌ ERROR: Circular reference detected!\n`);

      for (const cycle of cycles) {
        console.error(`   Cycle: ${cycle.join(' → ')}`);
      }

      console.error(`\n   Sankey diagrams must have unidirectional flow.`);
      console.error(`   Fix: Use separate node names for return paths.`);
      console.error(`   Example: "Core Services" → "Core Services (Updates)"\n`);

      hasErrors = true;
    } else {
      console.log(`   ✅ No circular references\n`);
    }
  }

  return !hasErrors;
}

// Main
const args = process.argv.slice(2);

if (args.length === 0) {
  console.error('Usage: node validate-mermaid-sankey.js <file.md>');
  process.exit(1);
}

const filePath = args[0];
const isValid = validateFile(filePath);

if (!isValid) {
  console.error('\n❌ Validation failed\n');
  process.exit(1);
}

console.log('✅ All Sankey diagrams are valid!\n');
process.exit(0);
