import React from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line, PieChart, Pie, Cell } from 'recharts';

const data = [
  { name: 'Mon', proofs: 40, errors: 2 },
  { name: 'Tue', proofs: 30, errors: 1 },
  { name: 'Wed', proofs: 55, errors: 3 },
  { name: 'Thu', proofs: 80, errors: 5 },
  { name: 'Fri', proofs: 65, errors: 2 },
  { name: 'Sat', proofs: 25, errors: 0 },
  { name: 'Sun', proofs: 15, errors: 0 },
];

const pieData = [
  { name: 'Success', value: 924 },
  { name: 'Failed', value: 12 },
  { name: 'Pending', value: 45 },
];

const COLORS = ['#10B981', '#EF4444', '#F59E0B'];

export default function Analytics() {
  return (
    <div className="space-y-8 h-full">
       <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Analytics</h1>
        <p className="text-gray-500 dark:text-gray-400">Deep dive into your usage metrics.</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        
        {/* Proof Volume */}
        <div className="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
           <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">Proof Generation Volume</h3>
           <div className="h-72">
             <ResponsiveContainer width="100%" height="100%">
               <BarChart data={data}>
                 <CartesianGrid strokeDasharray="3 3" opacity={0.1} />
                 <XAxis dataKey="name" stroke="#888888" fontSize={12} tickLine={false} axisLine={false} />
                 <YAxis stroke="#888888" fontSize={12} tickLine={false} axisLine={false} tickFormatter={(value) => `${value}`} />
                 <Tooltip 
                    contentStyle={{ borderRadius: '8px', border: 'none', boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)' }}
                 />
                 <Bar dataKey="proofs" fill="#3B82F6" radius={[4, 4, 0, 0]} />
               </BarChart>
             </ResponsiveContainer>
           </div>
        </div>

        {/* Success Rate */}
        <div className="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
           <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">Success vs Failure</h3>
           <div className="h-72">
             <ResponsiveContainer width="100%" height="100%">
               <PieChart>
                  <Pie
                    data={pieData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={80}
                    paddingAngle={5}
                    dataKey="value"
                  >
                    {pieData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip />
               </PieChart>
             </ResponsiveContainer>
             <div className="flex justify-center gap-6 mt-4">
                {pieData.map((entry, index) => (
                  <div key={index} className="flex items-center text-sm text-gray-500">
                    <div className="w-3 h-3 rounded-full mr-2" style={{ backgroundColor: COLORS[index] }} />
                    {entry.name} ({Math.round(entry.value / (924+12+45) * 100)}%)
                  </div>
                ))}
            </div>
           </div>
        </div>

      </div>
    </div>
  );
}
