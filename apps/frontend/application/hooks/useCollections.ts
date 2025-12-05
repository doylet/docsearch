import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useDependencyContainer } from "../providers/DependencyContainer";

export function useCollections() {
  const { collectionsUseCase } = useDependencyContainer();

  return useQuery({
    queryKey: ["collections"],
    queryFn: () => collectionsUseCase.listCollections(),
  });
}

export function useCreateCollection() {
  const { collectionsUseCase } = useDependencyContainer();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => collectionsUseCase.createCollection(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["collections"] });
    },
  });
}

export function useDeleteCollection() {
  const { collectionsUseCase } = useDependencyContainer();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => collectionsUseCase.deleteCollection(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["collections"] });
    },
  });
}
