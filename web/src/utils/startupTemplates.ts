import { StartupTemplate } from '../types';

/**
 * Predefined startup command templates for different runtimes
 */
export const STARTUP_TEMPLATES: StartupTemplate[] = [
  {
    name: 'Java JAR',
    runtime_type: 'java',
    template: 'java -Xmx{MEMORY}G -jar {JAR_FILE}',
    variables: ['MEMORY', 'JAR_FILE'],
  },
  {
    name: 'Java JAR with Arguments',
    runtime_type: 'java',
    template: 'java -Xmx{MEMORY}G -Xms{MIN_MEMORY}G -jar {JAR_FILE} {ARGS}',
    variables: ['MEMORY', 'MIN_MEMORY', 'JAR_FILE', 'ARGS'],
  },
  {
    name: '.NET Application',
    runtime_type: 'dotnet',
    template: 'dotnet {DLL_FILE}',
    variables: ['DLL_FILE'],
  },
  {
    name: '.NET with Arguments',
    runtime_type: 'dotnet',
    template: 'dotnet {DLL_FILE} {ARGS}',
    variables: ['DLL_FILE', 'ARGS'],
  },
  {
    name: 'Node.js Application',
    runtime_type: 'nodejs',
    template: 'node {MAIN_FILE}',
    variables: ['MAIN_FILE'],
  },
  {
    name: 'Node.js with Environment',
    runtime_type: 'nodejs',
    template: 'NODE_ENV={ENV} node {MAIN_FILE}',
    variables: ['ENV', 'MAIN_FILE'],
  },
  {
    name: 'Python Application',
    runtime_type: 'python',
    template: 'python {MAIN_FILE}',
    variables: ['MAIN_FILE'],
  },
  {
    name: 'Python with Virtual Environment',
    runtime_type: 'python',
    template: 'python -m venv venv && source venv/bin/activate && python {MAIN_FILE}',
    variables: ['MAIN_FILE'],
  },
  {
    name: 'Direct Binary',
    runtime_type: 'binary',
    template: './{BINARY_FILE}',
    variables: ['BINARY_FILE'],
  },
  {
    name: 'Binary with Arguments',
    runtime_type: 'binary',
    template: './{BINARY_FILE} {ARGS}',
    variables: ['BINARY_FILE', 'ARGS'],
  },
];

/**
 * Gets templates for a specific runtime type
 * @param runtimeType The runtime type to filter by
 * @returns Array of templates for the runtime type
 */
export function getTemplatesForRuntime(runtimeType: string): StartupTemplate[] {
  return STARTUP_TEMPLATES.filter(t => t.runtime_type === runtimeType);
}

/**
 * Fills in template variables with provided values
 * @param template The template string
 * @param variables Object mapping variable names to values
 * @returns The filled template string
 */
export function fillTemplate(template: string, variables: Record<string, string>): string {
  let result = template;

  for (const [key, value] of Object.entries(variables)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), value);
  }

  return result;
}

/**
 * Extracts variable placeholders from a template
 * @param template The template string
 * @returns Array of variable names found in the template
 */
export function extractVariables(template: string): string[] {
  const matches = template.match(/\{([^}]+)\}/g);
  if (!matches) return [];

  return matches.map(m => m.replace(/[{}]/g, ''));
}

/**
 * Validates that all required variables are filled
 * @param template The template string
 * @param variables Object mapping variable names to values
 * @returns Object with isValid boolean and missing array of missing variables
 */
export function validateTemplate(
  template: string,
  variables: Record<string, string>
): { isValid: boolean; missing: string[] } {
  const required = extractVariables(template);
  const missing = required.filter(v => !variables[v] || variables[v].trim() === '');

  return {
    isValid: missing.length === 0,
    missing,
  };
}

/**
 * Suggests files from archive based on runtime type
 * @param files Array of file paths from archive
 * @param runtimeType The runtime type
 * @returns Suggested file path or null
 */
export function suggestMainFile(files: string[], runtimeType: string): string | null {
  const lowerFiles = files.map(f => ({ original: f, lower: f.toLowerCase() }));

  switch (runtimeType) {
    case 'java':
      return lowerFiles.find(f => f.lower.endsWith('.jar'))?.original || null;

    case 'dotnet':
      return lowerFiles.find(f => f.lower.endsWith('.dll') && !f.lower.includes('lib'))?.original || null;

    case 'nodejs':
      // Look for common entry points
      const nodeEntries = ['index.js', 'main.js', 'app.js', 'server.js'];
      for (const entry of nodeEntries) {
        const found = lowerFiles.find(f => f.lower.endsWith(entry));
        if (found) return found.original;
      }
      return lowerFiles.find(f => f.lower.endsWith('.js'))?.original || null;

    case 'python':
      // Look for common entry points
      const pyEntries = ['main.py', 'app.py', '__main__.py', 'run.py'];
      for (const entry of pyEntries) {
        const found = lowerFiles.find(f => f.lower.endsWith(entry));
        if (found) return found.original;
      }
      return lowerFiles.find(f => f.lower.endsWith('.py'))?.original || null;

    case 'binary':
      return lowerFiles.find(f => f.lower.endsWith('.exe'))?.original || null;

    default:
      return null;
  }
}
