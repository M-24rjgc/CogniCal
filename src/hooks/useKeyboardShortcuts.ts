import { createContext, useContext } from 'react';

const noop = () => {};

export interface KeyboardShortcutContextValue {
  isCommandPaletteOpen: boolean;
  openCommandPalette: () => void;
  closeCommandPalette: () => void;
  toggleCommandPalette: () => void;
  isShortcutHelpOpen: boolean;
  openShortcutHelp: () => void;
  closeShortcutHelp: () => void;
  isHelpCenterOpen: boolean;
  openHelpCenter: () => void;
  closeHelpCenter: () => void;
  triggerFocusSearch: () => void;
}

export const FOCUS_SEARCH_EVENT_NAME = 'app:focus-search';

const defaultShortcutContext: KeyboardShortcutContextValue = {
  isCommandPaletteOpen: false,
  openCommandPalette: noop,
  closeCommandPalette: noop,
  toggleCommandPalette: noop,
  isShortcutHelpOpen: false,
  openShortcutHelp: noop,
  closeShortcutHelp: noop,
  isHelpCenterOpen: false,
  openHelpCenter: noop,
  closeHelpCenter: noop,
  triggerFocusSearch: noop,
};

export const KeyboardShortcutContext =
  createContext<KeyboardShortcutContextValue>(defaultShortcutContext);

export function useKeyboardShortcuts(): KeyboardShortcutContextValue {
  return useContext(KeyboardShortcutContext);
}
