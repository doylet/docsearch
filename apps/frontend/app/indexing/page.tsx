'use client';

import { useState } from 'react';
import { API_BASE_URL } from '@/infrastructure/config/apiConfig';

interface BrowseItem {
  name: string;
  path: string;
  is_directory: boolean;
  size?: number | null;
}

interface BrowseResponse {
  /** Listed directory; null when the response lists the allowed roots */
  path: string | null;
  /** Parent directory; null at a root or when listing the roots */
  parent_path: string | null;
  items: BrowseItem[];
  truncated: boolean;
}

interface IndexResponse {
  documents_processed: number;
  documents_skipped: number;
  processing_time_ms: number;
  status: string;
  message?: string;
}

const browseErrorMessage = (status: number): string => {
  switch (status) {
    case 403:
      return '✗ That directory is outside the allowed browse roots';
    case 404:
      return '✗ Directory not found';
    case 400:
      return '✗ That path is not a directory';
    default:
      return '✗ Error browsing directory';
  }
};

export default function IndexPage() {
  const [path, setPath] = useState('');
  const [collection, setCollection] = useState('');
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [showBrowser, setShowBrowser] = useState(false);
  const [browseData, setBrowseData] = useState<BrowseResponse | null>(null);

  /** Browse a directory, or list the allowed roots when no path is given */
  const browseDirectory = async (dirPath?: string) => {
    const query = dirPath ? `?path=${encodeURIComponent(dirPath)}` : '';
    try {
      const response = await fetch(`${API_BASE_URL}/api/browse${query}`);
      if (response.ok) {
        const data: BrowseResponse = await response.json();
        setBrowseData(data);
      } else {
        setMessage(browseErrorMessage(response.status));
      }
    } catch {
      setMessage('✗ Error browsing directory');
    }
  };

  const handleIndex = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setMessage('');

    try {
      // Indexing runs synchronously: the response arrives when indexing has finished
      const response = await fetch(`${API_BASE_URL}/api/index`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          path,
          collection,
        }),
      });

      if (response.ok) {
        const data: IndexResponse = await response.json();
        const skipped = data.documents_skipped > 0 ? `, skipped ${data.documents_skipped}` : '';
        setMessage(`✓ Successfully indexed ${data.documents_processed} documents${skipped}`);
        setPath('');
        setCollection('');
      } else {
        const error = await response.text();
        setMessage(`✗ Error: ${error}`);
      }
    } catch (error) {
      setMessage(`✗ Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  const selectPath = (selectedPath: string) => {
    setPath(selectedPath);
    setShowBrowser(false);
  };

  const listingRoots = browseData !== null && browseData.path === null;

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow-md p-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-6">Index Documents</h1>

        <form onSubmit={handleIndex} className="space-y-6">
          <div>
            <label htmlFor="collection" className="block text-sm font-medium text-gray-700 mb-2">
              Collection Name
            </label>
            <input
              id="collection"
              type="text"
              value={collection}
              onChange={(e) => setCollection(e.target.value)}
              placeholder="e.g., zero_latency_docs"
              required
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors text-gray-900 placeholder-gray-400"
            />
          </div>

          <div>
            <label htmlFor="path" className="block text-sm font-medium text-gray-700 mb-2">
              Document Path
            </label>

            <div className="flex gap-2">
              <input
                id="path"
                type="text"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="/path/to/documents"
                required
                className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors text-gray-900 placeholder-gray-400"
              />
              <button
                type="button"
                onClick={() => {
                  setShowBrowser(!showBrowser);
                  if (!showBrowser) {
                    browseDirectory(browseData?.path ?? undefined);
                  }
                }}
                className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
              >
                📂 Browse
              </button>
            </div>

            {/* Directory Browser */}
            {showBrowser && browseData && (
              <div className="mt-3 border border-gray-300 rounded-lg p-4 bg-gray-50">
                <div className="flex items-center gap-2 mb-3">
                  <span className="text-sm font-medium text-gray-700">
                    {listingRoots ? 'Allowed folders' : `Current: ${browseData.path}`}
                  </span>
                  {!listingRoots && (
                    <button
                      type="button"
                      onClick={() => browseDirectory(browseData.parent_path ?? undefined)}
                      className="text-sm text-blue-600 hover:text-blue-800"
                    >
                      {browseData.parent_path ? '↑ Up' : '↑ Allowed folders'}
                    </button>
                  )}
                </div>

                {listingRoots && browseData.items.length === 0 && (
                  <p className="text-sm text-gray-600">
                    Browsing is disabled. Set <code>DOC_INDEXER_BROWSE_ROOTS</code> on the server to
                    enable it, or type a path above.
                  </p>
                )}

                <div className="max-h-64 overflow-y-auto">
                  <div className="space-y-1">
                    {browseData.items.map((item) =>
                      item.is_directory ? (
                        <div key={item.path} className="flex items-center gap-2 p-2 hover:bg-gray-100 rounded">
                          <button
                            type="button"
                            onClick={() => browseDirectory(item.path)}
                            className="flex-1 text-left text-sm text-gray-700"
                          >
                            📁 {item.name}
                          </button>
                          <button
                            type="button"
                            onClick={() => selectPath(item.path)}
                            className="px-2 py-1 text-xs bg-blue-500 text-white rounded hover:bg-blue-600"
                          >
                            Select
                          </button>
                        </div>
                      ) : (
                        <div key={item.path} className="p-2 text-sm text-gray-500">
                          📄 {item.name}
                        </div>
                      )
                    )}
                  </div>
                </div>

                {browseData.truncated && (
                  <p className="mt-2 text-xs text-gray-500">
                    Showing the first {browseData.items.length} entries.
                  </p>
                )}
              </div>
            )}

            <p className="mt-2 text-sm text-gray-500">
              Enter the full path to the directory containing your documents, or use the browse button to navigate
            </p>
          </div>

          <button
            type="submit"
            disabled={loading}
            className="w-full bg-blue-600 text-white py-3 px-6 rounded-lg font-medium hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {loading ? 'Indexing...' : 'Index Documents'}
          </button>
        </form>

        {loading && (
          <div className="mt-6 p-4 rounded-lg bg-blue-50 border border-blue-200">
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
              <span className="text-sm font-medium text-blue-800">Indexing documents...</span>
            </div>
          </div>
        )}

        {message && (
          <div className={`mt-6 p-4 rounded-lg ${
            message.startsWith('✓')
              ? 'bg-green-50 text-green-800 border border-green-200'
              : 'bg-red-50 text-red-800 border border-red-200'
          }`}>
            {message}
          </div>
        )}
      </div>
    </div>
  );
}
