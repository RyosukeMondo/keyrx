/**
 * Custom hook for managing drag-and-drop state in the configuration editor.
 *
 * This hook encapsulates the state management and event handlers for the
 * drag-and-drop key mapping interface, including:
 * - Tracking the currently dragged key
 * - Handling drop events and creating mappings
 * - Auto-saving changes to the API with optimistic updates
 * - Error handling with rollback on failure
 *
 * @example
 * ```tsx
 * function ConfigPage() {
 *   const { activeDragKey, handleDragStart, handleDragEnd, handleKeyDrop } =
 *     useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' });
 *
 *   return (
 *     <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
 *       <DragKeyPalette />
 *       <KeyboardVisualizer onKeyDrop={handleKeyDrop} />
 *     </DndContext>
 *   );
 * }
 * ```
 */

import { useState, useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { DragStartEvent, DragEndEvent } from '@dnd-kit/core';
import * as configApi from '../api/config';
import { queryKeys } from '../lib/queryClient';
import type { AssignableKey, KeyMapping } from '../types/config';

interface UseDragAndDropOptions {
  /** Profile name to save mappings to */
  profileName: string;
  /** Active layer for layer-specific mappings */
  selectedLayer: string;
}

interface UseDragAndDropReturn {
  /** Currently dragged key (null if not dragging) */
  activeDragKey: AssignableKey | null;
  /** Handler for drag start event */
  handleDragStart: (event: DragStartEvent) => void;
  /** Handler for drag end event */
  handleDragEnd: (event: DragEndEvent) => void;
  /** Handler for key drop event */
  handleKeyDrop: (keyCode: string, droppedKey: AssignableKey) => Promise<void>;
  /** Whether a save operation is in progress */
  isSaving: boolean;
}

/**
 * Hook for managing drag-and-drop state and operations.
 */
export function useDragAndDrop({
  profileName,
  selectedLayer,
}: UseDragAndDropOptions): UseDragAndDropReturn {
  const queryClient = useQueryClient();
  const [activeDragKey, setActiveDragKey] = useState<AssignableKey | null>(null);

  // Mutation for saving key mappings with optimistic updates
  const saveMappingMutation = useMutation({
    mutationFn: ({ key, mapping }: { key: string; mapping: KeyMapping }) =>
      configApi.setKeyMapping(profileName, key, mapping),

    onMutate: async ({ key, mapping }) => {
      // Cancel outgoing queries to prevent race conditions
      await queryClient.cancelQueries({
        queryKey: ['config', profileName],
      });

      // Snapshot previous config for rollback
      const previousConfig = queryClient.getQueryData([
        'config',
        profileName,
      ]);

      // Optimistically update cache
      queryClient.setQueryData(['config', profileName], (old: any) => {
        if (!old) return old;

        return {
          ...old,
          keyMappings: {
            ...old.keyMappings,
            [key]: mapping,
          },
        };
      });

      return { previousConfig };
    },

    onError: (_error, _variables, context) => {
      // Rollback on error
      if (context?.previousConfig) {
        queryClient.setQueryData(
          ['config', profileName],
          context.previousConfig
        );
      }
    },

    onSuccess: () => {
      // Invalidate config query to refetch from server
      queryClient.invalidateQueries({
        queryKey: ['config', profileName],
      });
    },
  });

  /**
   * Handle drag start - store the dragged key
   */
  const handleDragStart = useCallback((event: DragStartEvent) => {
    const draggedKey = event.active.data.current as AssignableKey | undefined;
    if (draggedKey) {
      setActiveDragKey(draggedKey);
    }
  }, []);

  /**
   * Handle drag end - clear the dragged key
   */
  const handleDragEnd = useCallback((_event: DragEndEvent) => {
    setActiveDragKey(null);
  }, []);

  /**
   * Handle key drop - create mapping and save to API
   */
  const handleKeyDrop = useCallback(
    async (keyCode: string, droppedKey: AssignableKey) => {
      // Determine mapping type based on dropped key category
      let mapping: KeyMapping;

      switch (droppedKey.category) {
        case 'vk':
        case 'modifier':
        case 'lock':
          // Simple mapping - single key
          mapping = {
            keyCode,
            type: 'simple',
            simple: droppedKey.id,
          };
          break;

        case 'layer':
          // Layer switch mapping
          mapping = {
            keyCode,
            type: 'layer_switch',
            layer: droppedKey.id.replace('layer_', ''),
          };
          break;

        case 'macro':
          // Macro mapping - requires additional configuration
          // For now, create a placeholder that can be edited
          mapping = {
            keyCode,
            type: 'macro',
            macro: [droppedKey.id],
          };
          break;

        default:
          console.error('Unknown key category:', droppedKey.category);
          return;
      }

      // Save mapping via mutation (with optimistic update)
      await saveMappingMutation.mutateAsync({ key: keyCode, mapping });
    },
    [profileName, saveMappingMutation]
  );

  return {
    activeDragKey,
    handleDragStart,
    handleDragEnd,
    handleKeyDrop,
    isSaving: saveMappingMutation.isPending,
  };
}
