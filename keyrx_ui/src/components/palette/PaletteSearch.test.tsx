import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { PaletteSearch } from './PaletteSearch';
import { SearchMatch } from '../../hooks/usePaletteSearch';

// Mock search results for testing
const mockResults: SearchMatch[] = [
  {
    key: {
      id: 'KEY_A',
      label: 'A',
      description: 'Letter A key',
      category: 'basic',
      subcategory: 'letters',
      aliases: ['A', 'KC_A'],
    },
    score: 1000,
    matches: [
      {
        field: 'label',
        text: 'A',
        indices: [0],
      },
    ],
  },
  {
    key: {
      id: 'KEY_LALT',
      label: 'LAlt',
      description: 'Left Alt key',
      category: 'modifiers',
      subcategory: undefined,
      aliases: ['LALT', 'KC_LALT'],
    },
    score: 500,
    matches: [
      {
        field: 'label',
        text: 'LAlt',
        indices: [1],
      },
    ],
  },
  {
    key: {
      id: 'KEY_CAPS',
      label: 'CapsLock',
      description: 'Caps Lock key',
      category: 'special',
      subcategory: undefined,
      aliases: ['CAPS', 'KC_CAPS'],
    },
    score: 200,
    matches: [
      {
        field: 'label',
        text: 'CapsLock',
        indices: [1],
      },
    ],
  },
];

describe('PaletteSearch', () => {
  it('renders search input with placeholder', () => {
    render(
      <PaletteSearch
        value=""
        onChange={vi.fn()}
        results={[]}
        onSelect={vi.fn()}
        placeholder="Search test"
      />
    );

    expect(screen.getByPlaceholderText('Search test')).toBeInTheDocument();
  });

  it('displays search value in input', () => {
    render(
      <PaletteSearch
        value="test query"
        onChange={vi.fn()}
        results={[]}
        onSelect={vi.fn()}
      />
    );

    expect(screen.getByDisplayValue('test query')).toBeInTheDocument();
  });

  it('calls onChange when input value changes', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PaletteSearch
        value=""
        onChange={onChange}
        results={[]}
        onSelect={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox');
    await user.type(input, 'a');

    expect(onChange).toHaveBeenCalledWith('a');
  });

  it('shows clear button when value is not empty', () => {
    render(
      <PaletteSearch
        value="test"
        onChange={vi.fn()}
        results={[]}
        onSelect={vi.fn()}
      />
    );

    expect(screen.getByLabelText('Clear search')).toBeInTheDocument();
  });

  it('clears input when clear button is clicked', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PaletteSearch
        value="test"
        onChange={onChange}
        results={[]}
        onSelect={vi.fn()}
      />
    );

    await user.click(screen.getByLabelText('Clear search'));

    expect(onChange).toHaveBeenCalledWith('');
  });

  it('does not show dropdown when value is empty', () => {
    render(
      <PaletteSearch
        value=""
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
  });

  it('shows dropdown with results when value is not empty', () => {
    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    expect(screen.getByRole('listbox')).toBeInTheDocument();
    // Check by key IDs instead of labels (which might be highlighted)
    expect(screen.getByText('KEY_A')).toBeInTheDocument();
    expect(screen.getByText('KEY_LALT')).toBeInTheDocument();
    expect(screen.getByText('KEY_CAPS')).toBeInTheDocument();
  });

  it('displays result count', () => {
    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    expect(screen.getByText('3 results')).toBeInTheDocument();
  });

  it('displays singular "result" for single result', () => {
    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={[mockResults[0]]}
        onSelect={vi.fn()}
      />
    );

    expect(screen.getByText('1 result')).toBeInTheDocument();
  });

  it('shows "No results" message when results are empty', () => {
    render(
      <PaletteSearch
        value="xyz"
        onChange={vi.fn()}
        results={[]}
        onSelect={vi.fn()}
      />
    );

    expect(screen.getByText(/No results found for/)).toBeInTheDocument();
  });

  it('calls onSelect when result is clicked', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();

    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={onSelect}
      />
    );

    // Click the first result option
    const options = screen.getAllByRole('option');
    await user.click(options[0]);

    expect(onSelect).toHaveBeenCalledWith(mockResults[0]);
  });

  it('clears input after selecting a result', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PaletteSearch
        value="a"
        onChange={onChange}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    // Click the first result option
    const options = screen.getAllByRole('option');
    await user.click(options[0]);

    expect(onChange).toHaveBeenCalledWith('');
  });

  it('navigates results with ArrowDown key', async () => {
    const user = userEvent.setup();

    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox');
    await user.click(input);
    await user.keyboard('{ArrowDown}');

    // Second item should be selected (aria-selected="true")
    const options = screen.getAllByRole('option');
    expect(options[1]).toHaveAttribute('aria-selected', 'true');
  });

  it('navigates results with ArrowUp key', async () => {
    const user = userEvent.setup();

    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox');
    await user.click(input);
    await user.keyboard('{ArrowDown}');
    await user.keyboard('{ArrowDown}');
    await user.keyboard('{ArrowUp}');

    // Second item should be selected
    const options = screen.getAllByRole('option');
    expect(options[1]).toHaveAttribute('aria-selected', 'true');
  });

  it('selects result with Enter key', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();

    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={onSelect}
      />
    );

    const input = screen.getByRole('textbox');
    await user.click(input);
    await user.keyboard('{Enter}');

    expect(onSelect).toHaveBeenCalledWith(mockResults[0]);
  });

  it('clears input with Escape key', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PaletteSearch
        value="a"
        onChange={onChange}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox');
    await user.click(input);
    await user.keyboard('{Escape}');

    expect(onChange).toHaveBeenCalledWith('');
  });

  it('displays category badges for results', () => {
    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    expect(screen.getByText('basic')).toBeInTheDocument();
    expect(screen.getByText('modifiers')).toBeInTheDocument();
    expect(screen.getByText('special')).toBeInTheDocument();
  });

  it('highlights matched characters in results', () => {
    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    // Check that <mark> elements are present for highlighting
    const marks = screen.getAllByText('A', {
      selector: 'mark',
    });
    expect(marks.length).toBeGreaterThan(0);
  });

  it('applies compact styling when compact prop is true', () => {
    render(
      <PaletteSearch
        value=""
        onChange={vi.fn()}
        results={[]}
        onSelect={vi.fn()}
        compact={true}
      />
    );

    const input = screen.getByRole('textbox');
    expect(input).toHaveClass('text-xs');
  });

  it('has proper ARIA attributes', () => {
    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('aria-label', 'Search keys');
    expect(input).toHaveAttribute('aria-autocomplete', 'list');
    expect(input).toHaveAttribute('aria-controls', 'search-results');
    expect(input).toHaveAttribute('aria-expanded', 'true');
  });

  it('does not crash with empty results array', () => {
    expect(() => {
      render(
        <PaletteSearch
          value="test"
          onChange={vi.fn()}
          results={[]}
          onSelect={vi.fn()}
        />
      );
    }).not.toThrow();
  });

  it('handles keyboard navigation at boundaries', async () => {
    const user = userEvent.setup();

    render(
      <PaletteSearch
        value="a"
        onChange={vi.fn()}
        results={mockResults}
        onSelect={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox');
    await user.click(input);

    // Try to navigate up from first item (should stay at 0)
    await user.keyboard('{ArrowUp}');
    const options = screen.getAllByRole('option');
    expect(options[0]).toHaveAttribute('aria-selected', 'true');

    // Navigate to last item
    await user.keyboard('{ArrowDown}');
    await user.keyboard('{ArrowDown}');

    // Try to navigate down past last item (should stay at last)
    await user.keyboard('{ArrowDown}');
    expect(options[2]).toHaveAttribute('aria-selected', 'true');
  });
});
