import React from 'react';
import { Activity, Database, CheckCircle, Clock } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card';

export default function Overview() {
  const stats = [
    { label: 'Total Proofs', value: '1,284', icon: Activity, change: '+12% from last month' },
    { label: 'Success Rate', value: '99.4%', icon: CheckCircle, change: '+0.4% from last month' },
    { label: 'Storage Used', value: '45.2 GB', icon: Database, change: '+2.1 GB from last month' },
    { label: 'Avg Latency', value: '840ms', icon: Clock, change: '-120ms from last month' },
  ];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Overview</h1>
        <p className="text-gray-500 dark:text-gray-400">Welcome back to your enterprise dashboard.</p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat, index) => {
          const Icon = stat.icon;
          return (
            <div key={index} className="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between mb-4">
                <span className="text-gray-500 dark:text-gray-400 text-sm font-medium">{stat.label}</span>
                <span className="p-2 bg-blue-50 dark:bg-blue-900/20 rounded-lg text-blue-600 dark:text-blue-400">
                  <Icon className="w-5 h-5" />
                </span>
              </div>
              <div className="flex items-baseline justify-between">
                <h3 className="text-2xl font-bold text-gray-900 dark:text-white">{stat.value}</h3>
              </div>
              <p className="mt-2 text-xs text-green-600 dark:text-green-400 font-medium">
                {stat.change}
              </p>
            </div>
          );
        })}
      </div>

      {/* Recent Activity Placeholder */}
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Recent Activity (Last 24h)</h3>
        <div className="h-64 flex items-center justify-center border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-lg">
          <p className="text-gray-400">Activity chart will appear here</p>
        </div>
      </div>
    </div>
  );
}
