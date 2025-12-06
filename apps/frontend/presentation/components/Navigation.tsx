'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Search, FolderPlus } from 'lucide-react';

/**
 * Navigation Component
 *
 * Main navigation bar for the application.
 * Provides links to search and indexing pages.
 */
export function Navigation() {
  const pathname = usePathname();

  const navLinks = [
    {
      href: '/',
      label: 'Search',
      icon: Search,
      active: pathname === '/',
    },
    {
      href: '/indexing',
      label: 'Index',
      icon: FolderPlus,
      active: pathname === '/indexing',
    },
  ];

  return (
    <nav className="bg-white border-b border-gray-200">
      <div className="max-w-7xl mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center gap-8">
            <Link
              href="/"
              className="text-xl font-bold text-gray-900 hover:text-blue-600 transition-colors"
            >
              DocSearch
            </Link>

            <div className="flex gap-4">
              {navLinks.map((link) => {
                const Icon = link.icon;
                return (
                  <Link
                    key={link.href}
                    href={link.href}
                    className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-colors ${
                      link.active
                        ? 'bg-blue-50 text-blue-600 font-medium'
                        : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                    }`}
                  >
                    <Icon className="w-4 h-4" />
                    <span>{link.label}</span>
                  </Link>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </nav>
  );
}
