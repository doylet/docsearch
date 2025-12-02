import { useQuery } from "@tanstack/react-query";
import { useDependencyContainer } from "../providers/DependencyContainer";
import type { SearchOptions } from "@/domain/repositories/SearchRepository";

export function useSearch(query: string, options?: SearchOptions) {
  const { searchUseCase } = useDependencyContainer();

  return useQuery({
    queryKey: ["search", query, options],
    queryFn: () => searchUseCase.search(query, options),
    enabled: query.length >= 2,
    staleTime: 30000,
  });
}
