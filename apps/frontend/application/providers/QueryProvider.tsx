'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactNode, useState } from 'react';

/**
 * Query Provider
 *
 * Wraps the application with React Query's QueryClientProvider.
 * Configures default query options for data fetching and caching.
 */
export function QueryProvider({ children }: { children: ReactNode }) {
  // Create QueryClient inside component to avoid SSR issues
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // Stale time: 30 seconds (data is considered fresh)
            staleTime: 30 * 1000,
            // Cache time: 5 minutes (data remains in cache when unused)
            gcTime: 5 * 60 * 1000,
            // Retry failed requests once
            retry: 1,
            // Don't refetch on window focus (local backend won't change often)
            refetchOnWindowFocus: false,
          },
          mutations: {
            // Retry failed mutations once
            retry: 1,
          },
        },
      })
  );

  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}
