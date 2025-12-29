/**
 * TemplateLibrary - Browse and select macro templates
 *
 * This component displays a searchable library of pre-built macro templates
 * organized by category with filtering and preview capabilities.
 */

import { useState, useMemo } from 'react';
import type { MacroEvent } from '../hooks/useMacroRecorder';
import {
  getAllTemplates,
  getCategories,
  getTemplatesByCategory,
  searchTemplates,
  getTemplateMetadata,
  CATEGORY_INFO,
  type MacroTemplate,
  type TemplateCategory,
} from '../utils/macroTemplates';
import './TemplateLibrary.css';

interface TemplateLibraryProps {
  /** Callback when a template is selected */
  onSelectTemplate: (events: MacroEvent[], templateName: string) => void;
  /** Whether the library is currently active */
  isOpen?: boolean;
  /** Callback to close the library */
  onClose?: () => void;
}

/**
 * TemplateLibrary component.
 */
export function TemplateLibrary({ onSelectTemplate, isOpen = true, onClose }: TemplateLibraryProps) {
  const [selectedCategory, setSelectedCategory] = useState<TemplateCategory | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTemplate, setSelectedTemplate] = useState<MacroTemplate | null>(null);

  // Get filtered templates
  const filteredTemplates = useMemo(() => {
    let templates = getAllTemplates();

    // Filter by category
    if (selectedCategory !== 'all') {
      templates = getTemplatesByCategory(selectedCategory);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      templates = searchTemplates(searchQuery);
      // Further filter by category if selected
      if (selectedCategory !== 'all') {
        templates = templates.filter((t) => t.category === selectedCategory);
      }
    }

    return templates;
  }, [selectedCategory, searchQuery]);

  const handleSelectTemplate = (template: MacroTemplate) => {
    setSelectedTemplate(template);
  };

  const handleLoadTemplate = () => {
    if (selectedTemplate) {
      onSelectTemplate(selectedTemplate.events, selectedTemplate.name);
      if (onClose) {
        onClose();
      }
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div className="template-library">
      <div className="library-header">
        <h3>Macro Template Library</h3>
        {onClose && (
          <button onClick={onClose} className="btn-close-library">
            ×
          </button>
        )}
      </div>

      <div className="library-content">
        {/* Sidebar */}
        <div className="library-sidebar">
          <div className="search-box">
            <input
              type="text"
              placeholder="Search templates..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="search-input"
            />
          </div>

          <div className="category-list">
            <div className="category-header">Categories</div>
            <button
              className={`category-item ${selectedCategory === 'all' ? 'active' : ''}`}
              onClick={() => setSelectedCategory('all')}
            >
              <span className="category-name">All Templates</span>
              <span className="category-count">{getAllTemplates().length}</span>
            </button>
            {getCategories().map((category) => {
              const templates = getTemplatesByCategory(category);
              const info = CATEGORY_INFO[category];
              return (
                <button
                  key={category}
                  className={`category-item ${selectedCategory === category ? 'active' : ''}`}
                  onClick={() => setSelectedCategory(category)}
                >
                  <div className="category-info">
                    <span className="category-name">{info.name}</span>
                    <span className="category-description">{info.description}</span>
                  </div>
                  <span className="category-count">{templates.length}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Template List */}
        <div className="template-list">
          {filteredTemplates.length === 0 ? (
            <div className="no-templates">
              <p>No templates found</p>
              {searchQuery && <p className="hint">Try a different search term</p>}
            </div>
          ) : (
            filteredTemplates.map((template) => {
              const metadata = getTemplateMetadata(template);
              const isSelected = selectedTemplate?.id === template.id;
              return (
                <div
                  key={template.id}
                  className={`template-card ${isSelected ? 'selected' : ''}`}
                  onClick={() => handleSelectTemplate(template)}
                >
                  <div className="template-header">
                    <h4>{template.name}</h4>
                    <span className="template-category">{CATEGORY_INFO[template.category].name}</span>
                  </div>
                  <p className="template-description">{template.description}</p>
                  <div className="template-meta">
                    <span className="meta-item">{metadata.eventCount} events</span>
                    <span className="meta-item">~{metadata.estimatedDurationMs}ms</span>
                  </div>
                  <div className="template-tags">
                    {template.tags.map((tag) => (
                      <span key={tag} className="tag">
                        {tag}
                      </span>
                    ))}
                  </div>
                  {template.isTextSnippet && template.text && (
                    <div className="template-preview">
                      <code>{template.text.slice(0, 100)}{template.text.length > 100 ? '...' : ''}</code>
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Preview Panel */}
        {selectedTemplate && (
          <div className="template-preview-panel">
            <h4>Preview: {selectedTemplate.name}</h4>
            <p className="preview-description">{selectedTemplate.description}</p>

            {selectedTemplate.isTextSnippet && selectedTemplate.text && (
              <div className="preview-text">
                <h5>Text Content:</h5>
                <pre>{selectedTemplate.text}</pre>
              </div>
            )}

            <div className="preview-events">
              <h5>Events ({selectedTemplate.events.length}):</h5>
              <div className="events-preview-list">
                {selectedTemplate.events.slice(0, 10).map((event, index) => (
                  <div key={index} className="event-preview-item">
                    <span className="event-index">{index + 1}</span>
                    <span className="event-code">Code: {event.event.code}</span>
                    <span className="event-value">
                      {event.event.value === 1 ? 'Press' : 'Release'}
                    </span>
                    <span className="event-time">{(event.relative_timestamp_us / 1000).toFixed(2)}ms</span>
                  </div>
                ))}
                {selectedTemplate.events.length > 10 && (
                  <div className="event-preview-more">
                    ... and {selectedTemplate.events.length - 10} more events
                  </div>
                )}
              </div>
            </div>

            <div className="preview-actions">
              <button onClick={handleLoadTemplate} className="btn btn-primary">
                Load Template
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
