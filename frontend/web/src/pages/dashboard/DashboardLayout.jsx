import React, { useState } from 'react';
import { LayoutDashboard, Key, BarChart3, Settings } from 'lucide-react';

// Components (We will create these next)
import Overview from './Overview';
import ApiKeys from './ApiKeys';
import Analytics from './Analytics';

export default function DashboardLayout() {
  const [activeTab, setActiveTab] = useState('overview');

  const renderContent = () => {
    switch (activeTab) {
      case 'overview':
        return <Overview />;
      case 'keys':
        return <ApiKeys />;
      case 'analytics':
        return <Analytics />;
      default:
        return <Overview />;
    }
  };

  const navItems = [
    { id: 'overview', label: 'Overview', icon: LayoutDashboard },
    { id: 'keys', label: 'API Keys', icon: Key },
    { id: 'analytics', label: 'Analytics', icon: BarChart3 },
    { id: 'settings', label: 'Settings', icon: Settings },
  ];

  return (
    <div className="flex h-[calc(100vh-64px)] bg-gray-100 dark:bg-gray-900">
      {/* Sidebar */}
      <aside className="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
        <div className="p-6">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Enterprise</h2>
          <p className="text-sm text-gray-500 dark:text-gray-400">Admin Dashboard</p>
        </div>
        <nav className="px-4 space-y-2">
          {navItems.map((item) => {
             const Icon = item.icon;
             return (
              <button
                key={item.id}
                onClick={() => setActiveTab(item.id)}
                className={`flex items-center w-full px-4 py-3 text-sm font-medium rounded-lg transition-colors
                  ${activeTab === item.id 
                    ? 'bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-400' 
                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700'
                  }`}
              >
                <Icon className="w-5 h-5 mr-3" />
                {item.label}
              </button>
             );
          })}
        </nav>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto p-8">
        <div className="max-w-6xl mx-auto">
          {renderContent()}
        </div>
      </main>
    </div>
  );
}
