'use client';

import { useState, useEffect } from 'react';
import { API_BASE_URL } from '@/infrastructure/config/apiConfig';

interface BrowseItem {
  name: string;
  path: string;
  is_directory: boolean;
  size?: number;
}

interface BrowseResponse {
  path: string;
  parent_path?: string;
  items: BrowseItem[];
  common_paths?: string[];
}

interface IndexingProgress {
  id: string;
  total_files: number;
  processed_files: number;
  current_file?: string;
  status: 'starting' | 'scanning' | 'processing' | 'completed' | 'error';
  estimated_completion?: string;
  error_message?: string;
}

interface IndexResponse {
  documents_processed: number;
  processing_time_ms: number;
  status: string;
  message?: string;
  progress_id: string;
}

export default function IndexPage() {
  const [path, setPath] = useState('');
  const [collection, setCollection] = useState('');
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [showBrowser, setShowBrowser] = useState(false);
  const [browseData, setBrowseData] = useState<BrowseResponse | null>(null);
  const [currentBrowsePath, setCurrentBrowsePath] = useState('/Users/thomasdoyle');
  const [progress, setProgress] = useState<IndexingProgress | null>(null);
  const [progressId, setProgressId] = useState<string | null>(null);

  const commonPaths = [
    { name: 'Documents', path: '/Users/thomasdoyle/Documents' },
    { name: 'Downloads', path: '/Users/thomasdoyle/Downloads' },
    { name: 'Desktop', path: '/Users/thomasdoyle/Desktop' },
    { name: 'Daintree', path: '/Users/thomasdoyle/Daintree' },
    { name: 'Wilcannia', path: '/Users/thomasdoyle/Wilcannia' },
  ];

  const browseDirectory = async (dirPath: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/browse?path=${encodeURIComponent(dirPath)}`);
      if (response.ok) {
        const data: BrowseResponse = await response.json();
        setBrowseData(data);
        setCurrentBrowsePath(dirPath);
      } else {
        setMessage('✗ Error browsing directory');
      }
    } catch (error) {
      setMessage('✗ Error browsing directory');
    }
  };

  const pollProgress = async (progressId: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/progress?id=${progressId}`);
      if (response.ok) {
        const progressData: IndexingProgress | null = await response.json();
        if (progressData) {
          setProgress(progressData);

          // Continue polling if not completed or errored
          if (progressData.status === 'processing' || progressData.status === 'scanning' || progressData.status === 'starting') {
            setTimeout(() => pollProgress(progressId), 1000); // Poll every second
          } else {
            // Completed or error - stop polling
            setLoading(false);
            if (progressData.status === 'completed') {
              setMessage(`✓ Successfully indexed ${progressData.processed_files} documents`);
              setPath('');
              setCollection('');
            } else if (progressData.status === 'error') {
              setMessage(`✗ Error: ${progressData.error_message || 'Unknown error'}`);
            }
            setProgressId(null);
            setProgress(null);
          }
        }
      }
    } catch (error) {
      console.error('Error polling progress:', error);
    }
  };

  const handleIndex = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setMessage('');
    setProgress(null);
    setProgressId(null);

    try {
      const response = await fetch(`${API_BASE_URL}/api/index`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          path,
          collection_id: collection,
        }),
      });

      if (response.ok) {
        const data: IndexResponse = await response.json();

        // Start polling for progress
        setProgressId(data.progress_id);
        pollProgress(data.progress_id);
      } else {
        const error = await response.text();
        setMessage(`✗ Error: ${error}`);
        setLoading(false);
      }
    } catch (error) {
      setMessage(`✗ Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setLoading(false);
    }
  };

  const selectPath = (selectedPath: string) => {
    setPath(selectedPath);
    setShowBrowser(false);
  };

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

            {/* Common Paths Quick Select */}
            <div className="mb-3">
              <p className="text-sm text-gray-600 mb-2">Quick select:</p>
              <div className="flex flex-wrap gap-2">
                {commonPaths.map((item) => (
                  <button
                    key={item.path}
                    type="button"
                    onClick={() => selectPath(item.path)}
                    className="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded-full hover:bg-blue-200 transition-colors"
                  >
                    📁 {item.name}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex gap-2">
              <input
                id="path"
                type="text"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="/Users/thomasdoyle/Documents"
                required
                className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors text-gray-900 placeholder-gray-400"
              />
              <button
                type="button"
                onClick={() => {
                  setShowBrowser(!showBrowser);
                  if (!showBrowser) {
                    browseDirectory(currentBrowsePath);
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
                  <span className="text-sm font-medium text-gray-700">Current: {browseData.path}</span>
                  {browseData.parent_path && (
                    <button
                      type="button"
                      onClick={() => browseDirectory(browseData.parent_path!)}
                      className="text-sm text-blue-600 hover:text-blue-800"
                    >
                      ↑ Up
                    </button>
                  )}
                </div>

                <div className="max-h-64 overflow-y-auto">
                  <div className="space-y-1">
                    {browseData.items.filter(item => item.is_directory).map((item) => (
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
                    ))}
                  </div>
                </div>
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

        {/* Progress Visualization */}
        {loading && progress && (
          <div className="mt-6 p-4 rounded-lg bg-blue-50 border border-blue-200">
            <div className="flex items-center gap-2 mb-2">
              <div className="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
              <span className="text-sm font-medium text-blue-800">
                {progress.status === 'starting' && 'Starting indexing...'}
                {progress.status === 'scanning' && 'Scanning files...'}
                {progress.status === 'processing' && 'Processing documents...'}
              </span>
            </div>

            {progress.total_files > 0 && (
              <div className="mb-3">
                <div className="flex justify-between text-sm text-gray-600 mb-1">
                  <span>Progress: {progress.processed_files} of {progress.total_files} files</span>
                  <span>{Math.round((progress.processed_files / progress.total_files) * 100)}%</span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${(progress.processed_files / progress.total_files) * 100}%` }}
                  ></div>
                </div>
              </div>
            )}

            {progress.current_file && (
              <div className="text-sm text-gray-600">
                <span className="font-medium">Current file:</span> {progress.current_file}
              </div>
            )}
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
