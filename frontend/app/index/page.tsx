"use client";

import { useState } from "react";
import { useCollections } from "@/application/hooks/useCollections";
import { useIndexPath, useIndexFile } from "@/application/hooks/useIndexing";
import { IndexingInterface } from "@/presentation/components/IndexingInterface";
import { IndexingProgress } from "@/presentation/components/IndexingProgress";
import { IndexingSummary } from "@/presentation/components/IndexingSummary";
import { ErrorBoundary } from "@/presentation/components/ErrorBoundary";
import type { IndexOperation } from "@/domain/entities/IndexOperation";

export default function IndexPage() {
  const [currentOperation, setCurrentOperation] = useState<IndexOperation | null>(null);
  const { data: collections } = useCollections();

  const indexPathMutation = useIndexPath();
  const indexFileMutation = useIndexFile();

  const isIndexing = indexPathMutation.isPending || indexFileMutation.isPending;
  const error = indexPathMutation.error || indexFileMutation.error;

  const handleIndexPath = async (path: string, collection?: string, recursive?: boolean) => {
    try {
      const result = await indexPathMutation.mutateAsync({
        path,
        options: {
          collection,
          recursive,
        },
      });

      setCurrentOperation(result);
    } catch (err) {
      console.error("Failed to index path:", err);
    }
  };

  const handleIndexFile = async (path: string, collection?: string) => {
    try {
      const result = await indexFileMutation.mutateAsync({
        path,
        options: {
          collection,
        },
      });

      setCurrentOperation(result);
    } catch (err) {
      console.error("Failed to index file:", err);
    }
  };

  const handleReset = () => {
    setCurrentOperation(null);
    indexPathMutation.reset();
    indexFileMutation.reset();
  };

  const showInterface = !currentOperation || currentOperation.status === 'pending';
  const showProgress = currentOperation && currentOperation.status === 'in_progress';
  const showSummary = currentOperation && (currentOperation.status === 'completed' || currentOperation.status === 'failed');

  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-gray-50 py-8 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-gray-900 mb-2">
              Index Documents
            </h1>
            <p className="text-gray-600">
              Add new documents to the search system by indexing files or directories
            </p>
          </div>

          <div className="space-y-6">
            {showInterface && (
              <IndexingInterface
                collections={collections}
                onIndexPath={handleIndexPath}
                onIndexFile={handleIndexFile}
                isIndexing={isIndexing}
                error={error}
              />
            )}

            {showProgress && (
              <IndexingProgress operation={currentOperation} />
            )}

            {showSummary && (
              <IndexingSummary
                operation={currentOperation}
                onReset={handleReset}
              />
            )}
          </div>
        </div>
      </div>
    </ErrorBoundary>
  );
}
