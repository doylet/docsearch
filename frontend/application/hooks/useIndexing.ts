import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useDependencyContainer } from "../providers/DependencyContainer";
import type { IndexOptions } from "@/domain/repositories/IndexRepository";

export function useIndexPath() {
  const { indexUseCase } = useDependencyContainer();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ path, options }: { path: string; options?: IndexOptions }) =>
      indexUseCase.indexPath(path, options),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["collections"] });
    },
  });
}

export function useIndexFile() {
  const { indexUseCase } = useDependencyContainer();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ path, options }: { path: string; options?: IndexOptions }) =>
      indexUseCase.indexFile(path, options),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["collections"] });
    },
  });
}
