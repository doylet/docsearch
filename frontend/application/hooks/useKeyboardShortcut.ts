import { useEffect } from 'react';

/**
 * Custom hook for handling keyboard shortcuts
 *
 * @param key - The key to listen for (e.g., 'k', 'Enter')
 * @param callback - Function to call when the shortcut is triggered
 * @param options - Configuration options
 */
export function useKeyboardShortcut(
  key: string,
  callback: () => void,
  options: {
    metaKey?: boolean;
    ctrlKey?: boolean;
    shiftKey?: boolean;
    altKey?: boolean;
  } = {}
) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const { metaKey = false, ctrlKey = false, shiftKey = false, altKey = false } = options;

      // Check if the key matches and all modifier keys are correct
      const keyMatches = event.key.toLowerCase() === key.toLowerCase();
      const metaMatches = metaKey === (event.metaKey || event.ctrlKey); // Cmd on Mac, Ctrl on Windows
      const ctrlMatches = ctrlKey === event.ctrlKey;
      const shiftMatches = shiftKey === event.shiftKey;
      const altMatches = altKey === event.altKey;

      if (keyMatches && metaMatches && ctrlMatches && shiftMatches && altMatches) {
        event.preventDefault();
        callback();
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [key, callback, options]);
}
