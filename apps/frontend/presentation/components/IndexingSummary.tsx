'use client';

import { IndexOperation } from '@/domain/entities/IndexOperation';
import { CheckCircle2, XCircle, AlertTriangle } from 'lucide-react';

interface IndexingSummaryProps {
  operation: IndexOperation | null;
  onReset: () => void;
}

/**
 * IndexingSummary Component
 *
 * Final summary display after indexing operation completes.
 * Part of User Story 3 (P3) - File and Directory Indexing.
 *
 * Features:
 * - Success/failure summary with statistics
 * - Per-file error messages (if any)
 * - Action to start new indexing operation
 * - Visual indicators for operation outcome
 */
export function IndexingSummary({ operation, onReset }: IndexingSummaryProps) {
  if (!operation || (operation.status !== 'completed' && operation.status !== 'failed')) {
    return null;
  }

  const isSuccess = operation.status === 'completed';
  const hasErrors = operation.error_count > 0;

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <div className="flex items-center gap-3 mb-4">
        {isSuccess ? (
          hasErrors ? (
            <AlertTriangle className="w-8 h-8 text-yellow-500" />
          ) : (
            <CheckCircle2 className="w-8 h-8 text-green-500" />
          )
        ) : (
          <XCircle className="w-8 h-8 text-red-500" />
        )}

        <div>
          <h3 className="text-xl font-semibold text-gray-900">
            {isSuccess
              ? hasErrors
                ? 'Indexing Completed with Warnings'
                : 'Indexing Completed Successfully'
              : 'Indexing Failed'
            }
          </h3>
          {operation.path && (
            <p className="text-sm text-gray-600 mt-1">
              Path: {operation.path}
            </p>
          )}
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="p-4 bg-blue-50 rounded-lg">
          <p className="text-sm text-gray-600 mb-1">Documents Processed</p>
          <p className="text-2xl font-bold text-blue-600">
            {operation.documents_processed}
          </p>
        </div>

        {hasErrors && (
          <div className="p-4 bg-red-50 rounded-lg">
            <p className="text-sm text-gray-600 mb-1">Errors</p>
            <p className="text-2xl font-bold text-red-600">
              {operation.error_count}
            </p>
          </div>
        )}

        {operation.collection && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <p className="text-sm text-gray-600 mb-1">Collection</p>
            <p className="text-lg font-semibold text-gray-900">
              {operation.collection}
            </p>
          </div>
        )}

        {operation.started_at && operation.completed_at && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <p className="text-sm text-gray-600 mb-1">Duration</p>
            <p className="text-lg font-semibold text-gray-900">
              {calculateDuration(operation.started_at, operation.completed_at)}
            </p>
          </div>
        )}
      </div>

      {operation.errors && operation.errors.length > 0 && (
        <div className="mb-6">
          <h4 className="text-sm font-semibold text-gray-900 mb-2">
            Error Details
          </h4>
          <div className="max-h-48 overflow-y-auto space-y-2">
            {operation.errors.map((error, index) => (
              <div
                key={index}
                className="p-3 bg-red-50 border-l-4 border-red-400 rounded text-sm"
              >
                <p className="text-red-800">{error}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex gap-3">
        <button
          onClick={onReset}
          className="flex-1 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
        >
          Index More Documents
        </button>
      </div>

      {isSuccess && !hasErrors && (
        <p className="mt-4 text-sm text-center text-gray-600">
          All documents have been successfully indexed and are now searchable.
        </p>
      )}

      {hasErrors && (
        <p className="mt-4 text-sm text-center text-yellow-700">
          Some files could not be indexed. Successfully indexed documents are searchable.
        </p>
      )}
    </div>
  );
}

function calculateDuration(startedAt: string, completedAt: string): string {
  const start = new Date(startedAt).getTime();
  const end = new Date(completedAt).getTime();
  const durationMs = end - start;
  const durationSeconds = Math.floor(durationMs / 1000);

  if (durationSeconds < 60) {
    return `${durationSeconds}s`;
  }

  const minutes = Math.floor(durationSeconds / 60);
  const seconds = durationSeconds % 60;
  return `${minutes}m ${seconds}s`;
}
