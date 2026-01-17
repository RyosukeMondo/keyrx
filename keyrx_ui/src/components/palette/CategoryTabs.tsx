import React from 'react';

interface Category {
  id: string;
  label: string;
  icon: string;
}

interface CategoryTabsProps {
  categories: Category[];
  activeCategory: string;
  onCategoryChange: (categoryId: string) => void;
  subcategories: string[];
  activeSubcategory: string | null;
  onSubcategoryChange: (subcategory: string | null) => void;
  compact: boolean;
  isSearching: boolean;
}

/**
 * Category tabs with optional subcategory filter pills
 */
export const CategoryTabs: React.FC<CategoryTabsProps> = ({
  categories,
  activeCategory,
  onCategoryChange,
  subcategories,
  activeSubcategory,
  onSubcategoryChange,
  compact,
  isSearching,
}) => {
  if (isSearching) return null;

  return (
    <>
      {/* Category Tabs */}
      <div
        className={`flex gap-1 border-b border-slate-700 overflow-x-auto ${
          compact ? 'mb-2' : 'mb-4'
        }`}
      >
        {categories.map((cat) => (
          <button
            key={cat.id}
            onClick={() => onCategoryChange(cat.id)}
            className={`font-medium transition-colors whitespace-nowrap flex items-center gap-1 ${
              compact ? 'px-2 py-1 text-xs' : 'px-3 py-2 text-sm'
            } ${
              activeCategory === cat.id
                ? 'text-primary-400 border-b-2 border-primary-400'
                : 'text-slate-400 hover:text-slate-300'
            }`}
          >
            {!compact && <span>{cat.icon}</span>}
            <span>{cat.label}</span>
          </button>
        ))}
      </div>

      {/* Subcategory Pills (for Basic and Layers categories) - hidden in compact mode */}
      {!compact && subcategories.length > 0 && (
        <div className="flex gap-2 mb-3 flex-wrap">
          <button
            onClick={() => onSubcategoryChange(null)}
            className={`px-3 py-1 text-xs rounded-full transition-colors ${
              activeSubcategory === null
                ? 'bg-primary-500 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
          >
            All
          </button>
          {subcategories.map((sub) => (
            <button
              key={sub}
              onClick={() => onSubcategoryChange(sub || null)}
              className={`px-3 py-1 text-xs rounded-full transition-colors capitalize ${
                activeSubcategory === sub
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              }`}
            >
              {sub}
            </button>
          ))}
        </div>
      )}
    </>
  );
};
