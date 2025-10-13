import React, { createContext, useContext, useEffect, useState } from 'react';
import { ThemeMode, ColorblindMode, ThemeConfig } from '../types';

interface ThemeContextType {
  themeConfig: ThemeConfig;
  setThemeMode: (mode: ThemeMode) => void;
  setColorblindMode: (mode: ColorblindMode) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [themeConfig, setThemeConfig] = useState<ThemeConfig>(() => {
    const savedMode = localStorage.getItem('theme-mode') as ThemeMode | null;
    const savedColorblind = localStorage.getItem('colorblind-mode') as ColorblindMode | null;
    return {
      mode: savedMode || 'dark',
      colorblindMode: savedColorblind || 'none',
    };
  });

  const getThemeClass = () => {
    if (themeConfig.colorblindMode === 'none') {
      return themeConfig.mode;
    }
    return `${themeConfig.colorblindMode}-${themeConfig.mode}`;
  };

  useEffect(() => {
    const root = document.documentElement;
    const themeClass = getThemeClass();

    // Remove all possible theme classes
    root.classList.remove('light', 'dark',
      'protanopia-light', 'protanopia-dark',
      'deuteranopia-light', 'deuteranopia-dark',
      'tritanopia-light', 'tritanopia-dark'
    );

    root.classList.add(themeClass);
    localStorage.setItem('theme-mode', themeConfig.mode);
    localStorage.setItem('colorblind-mode', themeConfig.colorblindMode);
  }, [themeConfig]);

  const setThemeMode = (mode: ThemeMode) => {
    setThemeConfig(prev => ({ ...prev, mode }));
  };

  const setColorblindMode = (colorblindMode: ColorblindMode) => {
    setThemeConfig(prev => ({ ...prev, colorblindMode }));
  };

  const toggleTheme = () => {
    setThemeMode(themeConfig.mode === 'light' ? 'dark' : 'light');
  };

  return (
    <ThemeContext.Provider value={{ themeConfig, setThemeMode, setColorblindMode, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider');
  }
  return context;
}
