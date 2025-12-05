'use client';

interface LoadingSpinnerProps {
  message?: string;
}

/**
 * LoadingSpinner Component
 *
 * Presentation component for loading states.
 * Reusable across multiple features.
 */
export function LoadingSpinner({ message = 'Loading...' }: LoadingSpinnerProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12" role="status" aria-live="polite">
      <div className="animate-spin h-12 w-12 border-4 border-blue-500 border-t-transparent rounded-full mb-4" />
      <p className="text-gray-600 text-lg">{message}</p>
    </div>
  );
}
