/**
 * Unit tests for TemplateLibrary component
 *
 * Tests template browsing, filtering, search, selection, and insertion functionality.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { TemplateLibrary } from './TemplateLibrary';
import type { MacroEvent } from '../hooks/useMacroRecorder';
import * as macroTemplates from '../utils/macroTemplates';

// Mock the macroTemplates module
vi.mock('../utils/macroTemplates', async () => {
  const actual = await vi.importActual<typeof macroTemplates>('../utils/macroTemplates');

  return {
    ...actual,
    getAllTemplates: vi.fn(() => []),
    getCategories: vi.fn(() => ['text', 'development', 'productivity'] as macroTemplates.TemplateCategory[]),
    getTemplatesByCategory: vi.fn((category: macroTemplates.TemplateCategory) => []),
    searchTemplates: vi.fn((query: string) => []),
    getTemplateMetadata: vi.fn((template: macroTemplates.MacroTemplate) => ({
      id: template.id,
      name: template.name,
      description: template.description,
      category: template.category,
      tags: template.tags,
      eventCount: template.events.length,
      estimatedDurationMs: Math.round(
        template.events.length > 0
          ? template.events[template.events.length - 1].relative_timestamp_us / 1000
          : 0
      ),
    })),
    CATEGORY_INFO: {
      text: { name: 'Text Snippets', description: 'Common text templates' },
      development: { name: 'Development', description: 'Code templates' },
      productivity: { name: 'Productivity', description: 'Workflow templates' },
      communication: { name: 'Communication', description: 'Email templates' },
      custom: { name: 'Custom', description: 'User-defined templates' },
    },
  };
});

describe('TemplateLibrary', () => {
  const mockOnSelectTemplate = vi.fn();
  const mockOnClose = vi.fn();

  const defaultMockTemplates: macroTemplates.MacroTemplate[] = [
    {
      id: 'test-template-1',
      name: 'Test Template 1',
      description: 'First test template',
      category: 'text',
      tags: ['test', 'sample'],
      isTextSnippet: true,
      text: 'Hello World',
      events: [
        { event: { type: 0, code: 1, value: 1 }, relative_timestamp_us: 0 },
        { event: { type: 0, code: 1, value: 0 }, relative_timestamp_us: 1000 },
      ],
    },
    {
      id: 'test-template-2',
      name: 'Test Template 2',
      description: 'Second test template for development',
      category: 'development',
      tags: ['code', 'dev'],
      isTextSnippet: false,
      events: [
        { event: { type: 0, code: 2, value: 1 }, relative_timestamp_us: 0 },
        { event: { type: 0, code: 2, value: 0 }, relative_timestamp_us: 2000 },
        { event: { type: 0, code: 3, value: 1 }, relative_timestamp_us: 3000 },
      ],
    },
    {
      id: 'test-template-3',
      name: 'Productivity Template',
      description: 'Template for productivity tasks',
      category: 'productivity',
      tags: ['productivity', 'workflow'],
      isTextSnippet: true,
      text: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam.',
      events: [
        { event: { type: 0, code: 4, value: 1 }, relative_timestamp_us: 0 },
        { event: { type: 0, code: 4, value: 0 }, relative_timestamp_us: 1500 },
      ],
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    // Reset the mocks to return default templates and proper filtering
    vi.mocked(macroTemplates.getAllTemplates).mockReturnValue(defaultMockTemplates);
    vi.mocked(macroTemplates.getTemplatesByCategory).mockImplementation((category) =>
      defaultMockTemplates.filter((t) => t.category === category)
    );
    vi.mocked(macroTemplates.searchTemplates).mockImplementation((query) =>
      defaultMockTemplates.filter(
        (t) =>
          t.name.toLowerCase().includes(query.toLowerCase()) ||
          t.description.toLowerCase().includes(query.toLowerCase()) ||
          t.tags.some((tag) => tag.toLowerCase().includes(query.toLowerCase()))
      )
    );
  });

  it('should render the template library when isOpen is true', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
        onClose={mockOnClose}
      />
    );

    expect(screen.getByText('Macro Template Library')).toBeInTheDocument();
  });

  it('should not render when isOpen is false', () => {
    const { container } = render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={false}
        onClose={mockOnClose}
      />
    );

    expect(container.firstChild).toBeNull();
  });

  it('should display all templates by default', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    expect(screen.getByText('Test Template 1')).toBeInTheDocument();
    expect(screen.getByText('Test Template 2')).toBeInTheDocument();
    expect(screen.getByText('Productivity Template')).toBeInTheDocument();
  });

  it('should display template names and descriptions', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    expect(screen.getByText('Test Template 1')).toBeInTheDocument();
    expect(screen.getByText('First test template')).toBeInTheDocument();
    expect(screen.getByText('Test Template 2')).toBeInTheDocument();
    expect(screen.getByText('Second test template for development')).toBeInTheDocument();
  });

  it('should display category information', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Category buttons should be present in sidebar
    expect(screen.getAllByText('Text Snippets').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Development').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Productivity').length).toBeGreaterThan(0);
  });

  it('should filter templates by category when category is selected', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Click on "Development" category
    const devCategoryButton = screen.getByRole('button', { name: /Development/i });
    fireEvent.click(devCategoryButton);

    // Only development template should be visible
    expect(screen.getByText('Test Template 2')).toBeInTheDocument();
    expect(screen.queryByText('Test Template 1')).not.toBeInTheDocument();
    expect(screen.queryByText('Productivity Template')).not.toBeInTheDocument();
  });

  it('should filter templates by search query', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    const searchInput = screen.getByPlaceholderText('Search templates...');
    fireEvent.change(searchInput, { target: { value: 'productivity' } });

    // Only productivity template should be visible
    expect(screen.getByText('Productivity Template')).toBeInTheDocument();
    expect(screen.queryByText('Test Template 1')).not.toBeInTheDocument();
    expect(screen.queryByText('Test Template 2')).not.toBeInTheDocument();
  });

  it('should display empty state when no templates match search', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    const searchInput = screen.getByPlaceholderText('Search templates...');
    fireEvent.change(searchInput, { target: { value: 'nonexistent template' } });

    expect(screen.getByText('No templates found')).toBeInTheDocument();
    expect(screen.getByText('Try a different search term')).toBeInTheDocument();
  });

  it('should select template when clicked', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    const templateCard = screen.getByText('Test Template 1').closest('.template-card');
    expect(templateCard).toBeInTheDocument();

    if (templateCard) {
      fireEvent.click(templateCard);
    }

    // Preview panel should show with template details
    expect(screen.getByText('Preview: Test Template 1')).toBeInTheDocument();
  });

  it('should display template preview when template is selected', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Select first template by finding all matching elements and clicking the first card
    const templateCards = screen.getAllByText('Test Template 1');
    const templateCard = templateCards[0].closest('.template-card');
    if (templateCard) {
      fireEvent.click(templateCard);
    }

    // Preview should show template name and description
    expect(screen.getByText('Preview: Test Template 1')).toBeInTheDocument();

    // Should show text content for text snippets
    expect(screen.getByText('Text Content:')).toBeInTheDocument();

    // Text appears in both preview and in the template card, so use getAllByText
    const helloWorldElements = screen.getAllByText('Hello World');
    expect(helloWorldElements.length).toBeGreaterThan(0);
  });

  it('should display event count and estimated duration', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Check metadata for templates - use getAllByText since these appear in template cards
    const twoEventElements = screen.getAllByText(/2 events/);
    const threeEventElements = screen.getAllByText(/3 events/);

    expect(twoEventElements.length).toBeGreaterThan(0); // Test Template 1
    expect(threeEventElements.length).toBeGreaterThan(0); // Test Template 2
  });

  it('should display template tags', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    expect(screen.getByText('test')).toBeInTheDocument();
    expect(screen.getByText('sample')).toBeInTheDocument();
    expect(screen.getByText('code')).toBeInTheDocument();
    expect(screen.getByText('dev')).toBeInTheDocument();
  });

  it('should show truncated text preview for long text snippets', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // The productivity template has text > 100 chars
    const productivityCard = screen.getByText('Productivity Template').closest('.template-card');

    // Should show truncated preview with ellipsis
    expect(productivityCard?.textContent).toContain('...');
  });

  it('should trigger onSelectTemplate callback when Load Template is clicked', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
        onClose={mockOnClose}
      />
    );

    // Select a template
    const templateCard = screen.getByText('Test Template 1').closest('.template-card');
    if (templateCard) {
      fireEvent.click(templateCard);
    }

    // Click Load Template button
    const loadButton = screen.getByRole('button', { name: /Load Template/i });
    fireEvent.click(loadButton);

    // Check that callback was called with correct arguments
    expect(mockOnSelectTemplate).toHaveBeenCalledTimes(1);
    expect(mockOnSelectTemplate).toHaveBeenCalledWith(
      expect.any(Array),
      'Test Template 1'
    );

    // Should also close the library
    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it('should close library when close button is clicked', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
        onClose={mockOnClose}
      />
    );

    const closeButton = screen.getByRole('button', { name: /×/ });
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it('should not show close button when onClose is not provided', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    const closeButton = screen.queryByRole('button', { name: /×/ });
    expect(closeButton).not.toBeInTheDocument();
  });

  it('should handle large number of templates', () => {
    // Mock a large template list
    const largeMockTemplates = Array.from({ length: 100 }, (_, i) => ({
      id: `template-${i}`,
      name: `Template ${i}`,
      description: `Description for template ${i}`,
      category: 'text' as macroTemplates.TemplateCategory,
      tags: ['test'],
      isTextSnippet: false,
      events: [
        { event: { type: 0, code: 1, value: 1 }, relative_timestamp_us: 0 },
      ],
    }));

    vi.mocked(macroTemplates.getAllTemplates).mockReturnValue(largeMockTemplates);

    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Should render without performance issues
    expect(screen.getByText('Template 0')).toBeInTheDocument();
    expect(screen.getByText('Template 99')).toBeInTheDocument();
  });

  it('should show limited event preview and indicate more events', () => {
    // Create a template with more than 10 events
    const templateWithManyEvents: macroTemplates.MacroTemplate = {
      id: 'many-events',
      name: 'Many Events Template',
      description: 'Template with many events',
      category: 'development',
      tags: ['test'],
      isTextSnippet: false,
      events: Array.from({ length: 15 }, (_, i) => ({
        event: { type: 0, code: i, value: i % 2 },
        relative_timestamp_us: i * 1000,
      })),
    };

    vi.mocked(macroTemplates.getAllTemplates).mockReturnValue([templateWithManyEvents]);

    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Select the template
    const templateCard = screen.getByText('Many Events Template').closest('.template-card');
    if (templateCard) {
      fireEvent.click(templateCard);
    }

    // Should show message about additional events
    expect(screen.getByText(/... and 5 more events/i)).toBeInTheDocument();
  });

  it('should display "All Templates" category with correct count', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Find the "All Templates" button in the category list
    const allTemplatesElement = screen.getByText('All Templates');
    expect(allTemplatesElement).toBeInTheDocument();

    // The button should contain both the text and the count (which is in a separate span)
    const button = allTemplatesElement.closest('button');
    expect(button?.textContent).toMatch(/All Templates.*3/);
  });

  it('should combine category filter with search query', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Select development category
    const devCategoryButton = screen.getByRole('button', { name: /Development/i });
    fireEvent.click(devCategoryButton);

    // Search for "template"
    const searchInput = screen.getByPlaceholderText('Search templates...');
    fireEvent.change(searchInput, { target: { value: 'template' } });

    // Should show only development templates matching search
    expect(screen.getByText('Test Template 2')).toBeInTheDocument();
    expect(screen.queryByText('Test Template 1')).not.toBeInTheDocument();
  });

  it('should apply selected styling to chosen template', () => {
    // Reset the mock to default templates
    const defaultTemplates: macroTemplates.MacroTemplate[] = [
      {
        id: 'test-template-1',
        name: 'Test Template 1',
        description: 'First test template',
        category: 'text',
        tags: ['test', 'sample'],
        isTextSnippet: true,
        text: 'Hello World',
        events: [
          { event: { type: 0, code: 1, value: 1 }, relative_timestamp_us: 0 },
          { event: { type: 0, code: 1, value: 0 }, relative_timestamp_us: 1000 },
        ],
      },
    ];

    vi.mocked(macroTemplates.getAllTemplates).mockReturnValue(defaultTemplates);

    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // Get the template card directly
    const templateCard = screen.getByText('Test Template 1').closest('.template-card');

    // Initially should not have selected class
    expect(templateCard).not.toHaveClass('selected');

    // Click to select
    if (templateCard) {
      fireEvent.click(templateCard);
    }

    // Should now have selected class
    expect(templateCard).toHaveClass('selected');
  });

  it('should not trigger onSelectTemplate if no template is selected', () => {
    render(
      <TemplateLibrary
        onSelectTemplate={mockOnSelectTemplate}
        isOpen={true}
      />
    );

    // No template selected, Load Template button should not be visible
    const loadButton = screen.queryByRole('button', { name: /Load Template/i });
    expect(loadButton).not.toBeInTheDocument();

    expect(mockOnSelectTemplate).not.toHaveBeenCalled();
  });
});
