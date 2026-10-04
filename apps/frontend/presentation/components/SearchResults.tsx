'use client';

import { SearchResult } from '@/domain/entities/SearchResult';
import { FileText, File, Clock, Database, Calendar, HardDrive, Tag } from 'lucide-react';
import { formatDistanceToNow, parseISO } from 'date-fns';

interface SearchResultsProps {
  results: SearchResult[];
}

// Utility functions for enhanced metadata display
const getFileExtension = (path: string): string => {
  const ext = path.split('.').pop()?.toLowerCase();
  return ext || '';
};

const getFileIcon = (ext: string) => {
  switch (ext) {
    case 'pdf':
      return <File className="w-5 h-5 text-red-500 mt-1 flex-shrink-0" />;
    case 'md':
    case 'markdown':
      return <FileText className="w-5 h-5 text-blue-500 mt-1 flex-shrink-0" />;
    case 'txt':
      return <FileText className="w-5 h-5 text-gray-500 mt-1 flex-shrink-0" />;
    case 'html':
    case 'htm':
      return <File className="w-5 h-5 text-orange-500 mt-1 flex-shrink-0" />;
    case 'doc':
    case 'docx':
      return <File className="w-5 h-5 text-blue-600 mt-1 flex-shrink-0" />;
    default:
      return <FileText className="w-5 h-5 text-gray-400 mt-1 flex-shrink-0" />;
  }
};

const formatFileSize = (bytes?: number): string => {
  if (!bytes || bytes === 0) return 'Unknown size';
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(1)} ${units[unitIndex]}`;
};

const getFreshnessIndicator = (indexedAt?: string): { label: string; className: string } => {
  if (!indexedAt) return { label: 'Unknown', className: 'text-gray-500' };

  try {
    const indexed = parseISO(indexedAt);
    const now = new Date();
    const daysDiff = (now.getTime() - indexed.getTime()) / (1000 * 60 * 60 * 24);

    if (daysDiff < 1) {
      return { label: 'Newly indexed', className: 'text-green-600' };
    } else if (daysDiff < 7) {
      return { label: 'Recently indexed', className: 'text-blue-600' };
    } else if (daysDiff < 30) {
      return { label: 'Indexed this month', className: 'text-orange-600' };
    } else {
      return { label: 'Older content', className: 'text-gray-600' };
    }
  } catch {
    return { label: 'Unknown', className: 'text-gray-500' };
  }
};

const formatDate = (dateString?: string): string => {
  if (!dateString) return 'Unknown';
  try {
    return formatDistanceToNow(parseISO(dateString), { addSuffix: true });
  } catch {
    return 'Unknown';
  }
};

export function SearchResults({ results }: SearchResultsProps) {
  return (
    <div className="space-y-4">
      {results.map((result) => {
        const extension = result.document.file_extension || getFileExtension(result.document.path);
        const freshnessIndicator = getFreshnessIndicator(result.document.indexed_at);

        return (
          <div
            key={result.document.document_id}
            className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow border border-gray-200"
          >
            <div className="flex items-start gap-3">
              {getFileIcon(extension)}
              <div className="flex-1 min-w-0">
                {/* Header with title and metadata badges */}
                <div className="flex items-start justify-between mb-2">
                  <div className="flex-1 min-w-0">
                    <h3 className="text-lg font-semibold text-gray-900 mb-1 truncate">
                      {result.document.title || result.document.file_name || result.document.path.split('/').pop()}
                    </h3>
                    <p className="text-sm text-gray-600 mb-2 truncate" title={result.document.path}>
                      {result.document.path}
                    </p>
                  </div>

                  {/* File type badge */}
                  {extension && (
                    <span className="ml-2 px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded uppercase font-mono">
                      .{extension}
                    </span>
                  )}
                </div>

                {/* Content preview */}
                {result.document.content && (
                  <p className="text-gray-700 mb-3 line-clamp-3">
                    {result.document.content.substring(0, 300)}
                    {result.document.content.length > 300 ? '...' : ''}
                  </p>
                )}

                {/* Highlights */}
                {result.highlights && result.highlights.length > 0 && (
                  <div className="mb-3 p-2 bg-yellow-50 border-l-4 border-yellow-400 rounded">
                    <p className="text-sm text-gray-700 italic">
                      {result.highlights[0]}
                    </p>
                  </div>
                )}

                {/* Enhanced metadata section */}
                <div className="space-y-2">
                  {/* Score and rank row */}
                  <div className="flex items-center gap-4 text-sm text-gray-500">
                    <span className="flex items-center gap-1">
                      <span className="font-medium">Score:</span>
                      <span className="text-blue-600 font-semibold">
                        {(result.score * 100).toFixed(1)}%
                      </span>
                    </span>

                    {result.rank && (
                      <span className="flex items-center gap-1">
                        <span className="font-medium">Rank:</span>
                        <span>#{result.rank}</span>
                      </span>
                    )}

                    <span className="flex items-center gap-1">
                      <Database className="w-3 h-3" />
                      <span className="font-medium">Collection:</span>
                      <span className="px-2 py-0.5 bg-blue-100 text-blue-800 rounded text-xs font-medium">
                        {result.document.collection}
                      </span>
                    </span>
                  </div>

                  {/* File metadata row */}
                  <div className="flex items-center gap-4 text-sm text-gray-500">
                    <span className="flex items-center gap-1">
                      <HardDrive className="w-3 h-3" />
                      <span className="font-medium">Size:</span>
                      <span>{formatFileSize(result.document.file_size)}</span>
                    </span>

                    {result.document.last_modified && (
                      <span className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        <span className="font-medium">Modified:</span>
                        <span>{formatDate(result.document.last_modified)}</span>
                      </span>
                    )}

                    {result.document.indexed_at && (
                      <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        <span className="font-medium">Indexed:</span>
                        <span className={freshnessIndicator.className}>
                          {formatDate(result.document.indexed_at)}
                        </span>
                      </span>
                    )}

                    <span className="flex items-center gap-1">
                      <Tag className="w-3 h-3" />
                      <span className={`text-xs px-2 py-1 rounded-full ${freshnessIndicator.className} bg-opacity-10`}>
                        {freshnessIndicator.label}
                      </span>
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
