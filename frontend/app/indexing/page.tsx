'use client';

import { useState } from 'react';
import { API_BASE_URL } from '@/infrastructure/config/apiConfig';

export default function IndexPage() {
  const [path, setPath] = useState('');
  const [collection, setCollection] = useState('');
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');

  const handleIndex = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setMessage('');

    try {
      const response = await fetch(`${API_BASE_URL}/api/index`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          path,
          collection_name: collection,
        }),
      });

      if (response.ok) {
        const data = await response.json();
        setMessage(`✓ Successfully indexed ${data.indexed_count || 0} documents`);
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
              placeholder="e.g., my-docs"
              required
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors text-gray-900 placeholder-gray-400"
            />
          </div>

          <div>
            <label htmlFor="path" className="block text-sm font-medium text-gray-700 mb-2">
              Document Path
            </label>
            <input
              id="path"
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="/path/to/your/docs"
              required
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors text-gray-900 placeholder-gray-400"
            />
            <p className="mt-2 text-sm text-gray-500">
              Enter the full path to the directory containing your documents
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
