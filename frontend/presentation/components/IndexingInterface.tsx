'use client';

import { useState } from 'react';
import { Collection } from '@/domain/entities/Collection';
import { FolderOpen, FileText } from 'lucide-react';

interface IndexingInterfaceProps {
  collections: Collection[] | undefined;
  onIndexPath: (path: string, collection?: string, recursive?: boolean) => void;
  onIndexFile: (path: string, collection?: string) => void;
  isIndexing: boolean;
  error: Error | null;
}

/**
 * IndexingInterface Component
 *
 * User interface for indexing files and directories.
 * Part of User Story 3 (P3) - File and Directory Indexing.
 *
 * Features:
 * - Directory path input with browser file picker
 * - Collection selector with "default" fallback
 * - Recursive indexing toggle
 * - Separate file and directory indexing modes
 * - Input validation and error display
 */
export function IndexingInterface({
  collections,
  onIndexPath,
  onIndexFile,
  isIndexing,
  error
}: IndexingInterfaceProps) {
  const [path, setPath] = useState('');
  const [collection, setCollection] = useState('default');
  const [recursive, setRecursive] = useState(true);
  const [mode, setMode] = useState<'directory' | 'file'>('directory');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!path.trim()) {
      return;
    }

    const selectedCollection = collection === 'default' ? undefined : collection;

    if (mode === 'directory') {
      onIndexPath(path.trim(), selectedCollection, recursive);
    } else {
      onIndexFile(path.trim(), selectedCollection);
    }
  };

  const handleFileSelect = async () => {
    try {
      // Use browser file picker API
      if ('showOpenFilePicker' in window) {
        const [fileHandle] = await (window as any).showOpenFilePicker({
          multiple: false,
        });
        const file = await fileHandle.getFile();
        setPath(file.name);
        setMode('file');
      }
    } catch (err) {
      // User cancelled or browser doesn't support API
      console.log('File picker cancelled or not supported');
    }
  };

  const handleDirectorySelect = async () => {
    try {
      // Use browser directory picker API
      if ('showDirectoryPicker' in window) {
        const directoryHandle = await (window as any).showDirectoryPicker();
        setPath(directoryHandle.name);
        setMode('directory');
      }
    } catch (err) {
      // User cancelled or browser doesn't support API
      console.log('Directory picker cancelled or not supported');
    }
  };

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">
        Index Documents
      </h2>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label htmlFor="indexing-mode" className="block text-sm font-medium text-gray-700 mb-2">
            Indexing Mode
          </label>
          <div className="flex gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="mode"
                value="directory"
                checked={mode === 'directory'}
                onChange={(e) => setMode(e.target.value as 'directory')}
                className="w-4 h-4 text-blue-600 focus:ring-blue-500"
              />
              <FolderOpen className="w-4 h-4" />
              <span className="text-sm text-gray-700">Directory</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="mode"
                value="file"
                checked={mode === 'file'}
                onChange={(e) => setMode(e.target.value as 'file')}
                className="w-4 h-4 text-blue-600 focus:ring-blue-500"
              />
              <FileText className="w-4 h-4" />
              <span className="text-sm text-gray-700">Single File</span>
            </label>
          </div>
        </div>

        <div>
          <label htmlFor="path-input" className="block text-sm font-medium text-gray-700 mb-2">
            {mode === 'directory' ? 'Directory Path' : 'File Path'}
          </label>
          <div className="flex gap-2">
            <input
              id="path-input"
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder={mode === 'directory' ? '/path/to/directory' : '/path/to/file.md'}
              disabled={isIndexing}
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed text-gray-900 placeholder:text-gray-400"
              aria-label={mode === 'directory' ? 'Directory path to index' : 'File path to index'}
            />
            <button
              type="button"
              onClick={mode === 'directory' ? handleDirectorySelect : handleFileSelect}
              disabled={isIndexing}
              className="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 disabled:bg-gray-100 disabled:cursor-not-allowed transition-colors"
              aria-label={mode === 'directory' ? 'Browse for directory' : 'Browse for file'}
            >
              Browse
            </button>
          </div>
          <p className="mt-1 text-xs text-gray-500">
            {mode === 'directory'
              ? 'Enter or browse to a directory containing documents to index'
              : 'Enter or browse to a single document file to index'
            }
          </p>
        </div>

        <div>
          <label htmlFor="collection-select" className="block text-sm font-medium text-gray-700 mb-2">
            Collection
          </label>
          <select
            id="collection-select"
            value={collection}
            onChange={(e) => setCollection(e.target.value)}
            disabled={isIndexing}
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white disabled:bg-gray-100 disabled:cursor-not-allowed"
            aria-label="Select collection for indexed documents"
          >
            <option value="default">Default Collection</option>
            {collections?.map((coll) => (
              <option key={coll.name} value={coll.name}>
                {coll.name}
              </option>
            ))}
          </select>
        </div>

        {mode === 'directory' && (
          <div className="flex items-center gap-2">
            <input
              id="recursive-checkbox"
              type="checkbox"
              checked={recursive}
              onChange={(e) => setRecursive(e.target.checked)}
              disabled={isIndexing}
              className="w-4 h-4 text-blue-600 focus:ring-blue-500 rounded disabled:cursor-not-allowed"
            />
            <label htmlFor="recursive-checkbox" className="text-sm text-gray-700 cursor-pointer">
              Index subdirectories recursively
            </label>
          </div>
        )}

        {error && (
          <div className="p-3 bg-red-50 border-l-4 border-red-400 rounded text-sm text-red-800">
            {error.message || 'Failed to start indexing operation'}
          </div>
        )}

        <button
          type="submit"
          disabled={isIndexing || !path.trim()}
          className="w-full px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors font-medium"
        >
          {isIndexing ? 'Indexing...' : `Index ${mode === 'directory' ? 'Directory' : 'File'}`}
        </button>
      </form>
    </div>
  );
}
