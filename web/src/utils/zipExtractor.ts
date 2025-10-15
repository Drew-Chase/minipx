import { unzip } from 'fflate';
import { ArchiveFile } from '../types';

/**
 * Extracts files from a zip archive
 * @param file The zip file to extract
 * @returns Promise resolving to array of ArchiveFile objects
 */
export async function extractZipArchive(file: File): Promise<ArchiveFile[]> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = async (e) => {
      try {
        const arrayBuffer = e.target?.result as ArrayBuffer;
        const uint8Array = new Uint8Array(arrayBuffer);

        unzip(uint8Array, (err, unzipped) => {
          if (err) {
            reject(err);
            return;
          }

          const files: ArchiveFile[] = [];

          for (const [path, content] of Object.entries(unzipped)) {
            // Skip directories
            if (path.endsWith('/')) continue;

            const fileName = path.split('/').pop() || path;
            const isExecutable = isExecutableFile(fileName);

            files.push({
              name: fileName,
              path: path,
              size: content.length,
              isExecutable,
            });
          }

          resolve(files);
        });
      } catch (error) {
        reject(error);
      }
    };

    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

/**
 * Checks if a file is an executable based on its extension
 * @param filename The filename to check
 * @returns true if the file is executable
 */
function isExecutableFile(filename: string): boolean {
  const executableExtensions = [
    '.exe',
    '.jar',
    '.dll',
    '.so',
    '.dylib',
    '.sh',
    '.bat',
    '.cmd',
    '.ps1',
    '.app',
    '.js',
    '.mjs',
    '.py',
    '.rb',
    '.go',
  ];

  const lowerFilename = filename.toLowerCase();
  return executableExtensions.some(ext => lowerFilename.endsWith(ext));
}

/**
 * Gets executable files from archive
 * @param files Array of archive files
 * @returns Filtered array of executable files
 */
export function getExecutableFiles(files: ArchiveFile[]): ArchiveFile[] {
  return files.filter(f => f.isExecutable);
}

/**
 * Gets files by extension
 * @param files Array of archive files
 * @param extensions Array of extensions to filter by (e.g., ['.jar', '.exe'])
 * @returns Filtered array of files
 */
export function getFilesByExtension(files: ArchiveFile[], extensions: string[]): ArchiveFile[] {
  return files.filter(f =>
    extensions.some(ext => f.name.toLowerCase().endsWith(ext.toLowerCase()))
  );
}
