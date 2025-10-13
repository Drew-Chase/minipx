import {heroui} from "@heroui/react";

/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
        "./node_modules/@heroui/theme/dist/**/*.{js,ts,jsx,tsx}"
    ],
    theme: {
        extend: {
            animation: {
                'fade-in': 'fadeIn 0.5s ease-in-out',
                'slide-up': 'slideUp 0.4s ease-out',
                'slide-down': 'slideDown 0.4s ease-out',
                'scale-in': 'scaleIn 0.3s ease-out',
                'bounce-subtle': 'bounceSubtle 0.6s ease-in-out',
            },
            keyframes: {
                fadeIn: {
                    '0%': { opacity: '0' },
                    '100%': { opacity: '1' },
                },
                slideUp: {
                    '0%': { transform: 'translateY(20px)', opacity: '0' },
                    '100%': { transform: 'translateY(0)', opacity: '1' },
                },
                slideDown: {
                    '0%': { transform: 'translateY(-20px)', opacity: '0' },
                    '100%': { transform: 'translateY(0)', opacity: '1' },
                },
                scaleIn: {
                    '0%': { transform: 'scale(0.9)', opacity: '0' },
                    '100%': { transform: 'scale(1)', opacity: '1' },
                },
                bounceSubtle: {
                    '0%, 100%': { transform: 'translateY(0)' },
                    '50%': { transform: 'translateY(-5px)' },
                },
            },
        },
    },
    darkMode: "class",
    plugins: [heroui({
        themes: {
            light: {
                colors: {
                    primary: {
                        DEFAULT: "#6366f1",
                        foreground: "#ffffff",
                        50: "#eef2ff",
                        100: "#e0e7ff",
                        200: "#c7d2fe",
                        300: "#a5b4fc",
                        400: "#818cf8",
                        500: "#6366f1",
                        600: "#4f46e5",
                        700: "#4338ca",
                        800: "#3730a3",
                        900: "#312e81",
                    },
                    secondary: {
                        DEFAULT: "#10b981",
                        foreground: "#ffffff",
                    },
                    success: "#10b981",
                    warning: "#f59e0b",
                    danger: "#ef4444",
                    background: "#f8fafc",
                    foreground: "#1e293b",
                }
            },
            dark: {
                colors: {
                    primary: {
                        DEFAULT: "#818cf8",
                        foreground: "#0f172a",
                        50: "#312e81",
                        100: "#3730a3",
                        200: "#4338ca",
                        300: "#4f46e5",
                        400: "#6366f1",
                        500: "#818cf8",
                        600: "#a5b4fc",
                        700: "#c7d2fe",
                        800: "#e0e7ff",
                        900: "#eef2ff",
                    },
                    secondary: {
                        DEFAULT: "#34d399",
                        foreground: "#0f172a",
                    },
                    success: "#34d399",
                    warning: "#fbbf24",
                    danger: "#f87171",
                    background: "#0f172a",
                    foreground: "#f1f5f9",
                }
            },
            // Colorblind-friendly themes
            "protanopia-light": {
                colors: {
                    primary: {
                        DEFAULT: "#0ea5e9",
                        foreground: "#ffffff",
                    },
                    secondary: "#f59e0b",
                    success: "#0ea5e9",
                    warning: "#f59e0b",
                    danger: "#0369a1",
                    background: "#f8fafc",
                    foreground: "#1e293b",
                }
            },
            "protanopia-dark": {
                colors: {
                    primary: {
                        DEFAULT: "#38bdf8",
                        foreground: "#0f172a",
                    },
                    secondary: "#fbbf24",
                    success: "#38bdf8",
                    warning: "#fbbf24",
                    danger: "#0ea5e9",
                    background: "#0f172a",
                    foreground: "#f1f5f9",
                }
            },
            "deuteranopia-light": {
                colors: {
                    primary: {
                        DEFAULT: "#8b5cf6",
                        foreground: "#ffffff",
                    },
                    secondary: "#f59e0b",
                    success: "#8b5cf6",
                    warning: "#f59e0b",
                    danger: "#6d28d9",
                    background: "#f8fafc",
                    foreground: "#1e293b",
                }
            },
            "deuteranopia-dark": {
                colors: {
                    primary: {
                        DEFAULT: "#a78bfa",
                        foreground: "#0f172a",
                    },
                    secondary: "#fbbf24",
                    success: "#a78bfa",
                    warning: "#fbbf24",
                    danger: "#8b5cf6",
                    background: "#0f172a",
                    foreground: "#f1f5f9",
                }
            },
            "tritanopia-light": {
                colors: {
                    primary: {
                        DEFAULT: "#ec4899",
                        foreground: "#ffffff",
                    },
                    secondary: "#06b6d4",
                    success: "#ec4899",
                    warning: "#06b6d4",
                    danger: "#be185d",
                    background: "#f8fafc",
                    foreground: "#1e293b",
                }
            },
            "tritanopia-dark": {
                colors: {
                    primary: {
                        DEFAULT: "#f472b6",
                        foreground: "#0f172a",
                    },
                    secondary: "#22d3ee",
                    success: "#f472b6",
                    warning: "#22d3ee",
                    danger: "#ec4899",
                    background: "#0f172a",
                    foreground: "#f1f5f9",
                }
            },
        }
    })]
}