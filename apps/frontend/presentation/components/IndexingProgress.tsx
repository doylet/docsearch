'use client';

import { IndexOperation } from '@/domain/entities/IndexOperation';
import { CheckCircle2, XCircle, Clock, Loader2 } from 'lucide-react';

interface IndexingProgressProps {
  operation: IndexOperation | null;
}

/**
 * IndexingProgress Component
 *
 * Real-time progress display for active indexing operations.
 * Part of User Story 3 (P3) - File and Directory Indexing.
 *
 * Features:
 * - Visual progress bar with percentage
 * - Documents processed counter
 * - Error count display
 * - Time elapsed calculation
 * - Status indicators (pending, in_progress, completed, failed, cancelled)
 */
export function IndexingProgress({ operation }: IndexingProgressProps) {
  if (!operation) {
    return null;
  }

  const { status, documents_processed, error_count, started_at, completed_at } = operation;

  const getStatusIcon = () => {
    switch (status) {
      case 'completed':
        return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case 'failed':
        return <XCircle className="w-5 h-5 text-red-500" />;
      case 'in_progress':
        return <Loader2 className="w-5 h-5 text-blue-500 animate-spin" />;
      case 'pending':
        return <Clock className="w-5 h-5 text-gray-400" />;
      case 'cancelled':
        return <XCircle className="w-5 h-5 text-gray-400" />;
    }
  };

  const getStatusColor = () => {
    switch (status) {
      case 'completed':
        return 'bg-green-50 border-green-200';
      case 'failed':
        return 'bg-red-50 border-red-200';
      case 'in_progress':
        return 'bg-blue-50 border-blue-200';
      case 'pending':
        return 'bg-gray-50 border-gray-200';
      case 'cancelled':
        return 'bg-gray-50 border-gray-200';
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'completed':
        return 'Completed';
      case 'failed':
        return 'Failed';
      case 'in_progress':
        return 'In Progress';
      case 'pending':
        return 'Pending';
      case 'cancelled':
        return 'Cancelled';
    }
  };

  const calculateTimeElapsed = () => {
    if (!started_at) return null;

    const endTime = completed_at ? new Date(completed_at).getTime() : Date.now();
    const startTime = new Date(started_at).getTime();
    const elapsedMs = endTime - startTime;
    const elapsedSeconds = Math.floor(elapsedMs / 1000);

    if (elapsedSeconds < 60) {
      return `${elapsedSeconds}s`;
    }

    const minutes = Math.floor(elapsedSeconds / 60);
    const seconds = elapsedSeconds % 60;
    return `${minutes}m ${seconds}s`;
  };

  const timeElapsed = calculateTimeElapsed();
  const progress = operation.progress || 0;

  return (
    <div className={`rounded-lg border-2 p-6 ${getStatusColor()}`}>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          {getStatusIcon()}
          <h3 className="text-lg font-semibold text-gray-900">
            {getStatusText()}
          </h3>
        </div>
        {timeElapsed && (
          <span className="text-sm text-gray-600">
            {timeElapsed}
          </span>
        )}
      </div>

      {status === 'in_progress' && (
        <div className="mb-4">
          <div className="flex justify-between text-sm text-gray-600 mb-1">
            <span>Progress</span>
            <span>{progress.toFixed(1)}%</span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${progress}%` }}
              role="progressbar"
              aria-valuenow={progress}
              aria-valuemin={0}
              aria-valuemax={100}
            />
          </div>
        </div>
      )}

      <div className="grid grid-cols-2 gap-4 text-sm">
        <div>
          <span className="text-gray-600">Documents Processed:</span>
          <span className="ml-2 font-semibold text-gray-900">
            {documents_processed}
          </span>
        </div>

        {error_count > 0 && (
          <div>
            <span className="text-gray-600">Errors:</span>
            <span className="ml-2 font-semibold text-red-600">
              {error_count}
            </span>
          </div>
        )}
      </div>

      {operation.path && (
        <div className="mt-3 pt-3 border-t border-gray-200">
          <p className="text-xs text-gray-600">
            <span className="font-medium">Path:</span> {operation.path}
          </p>
          {operation.collection && (
            <p className="text-xs text-gray-600 mt-1">
              <span className="font-medium">Collection:</span> {operation.collection}
            </p>
          )}
        </div>
      )}
    </div>
  );
}
