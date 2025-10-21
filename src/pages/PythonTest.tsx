import { useState } from 'react';
import { Button } from '../components/ui/button';
import { Badge } from '../components/ui/badge';
import {
  testConnection,
  getMemoryStatus,
  initializeMemory,
  searchMemory,
  storeConversation,
} from '../services/pythonMemory';

export default function PythonTestPage() {
  const [status, setStatus] = useState<any>(null);
  const [testResult, setTestResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const handleTestConnection = async () => {
    setLoading(true);
    try {
      const result = await testConnection();
      setTestResult(result);
    } catch (error) {
      setTestResult({ success: false, error: String(error) });
    }
    setLoading(false);
  };

  const handleGetStatus = async () => {
    setLoading(true);
    try {
      const result = await getMemoryStatus();
      setStatus(result);
    } catch (error) {
      setStatus({ error: String(error) });
    }
    setLoading(false);
  };

  const handleInitialize = async () => {
    setLoading(true);
    try {
      const result = await initializeMemory('./knowledge_base');
      setTestResult(result);
      // Refresh status
      const statusResult = await getMemoryStatus();
      setStatus(statusResult);
    } catch (error) {
      setTestResult({ success: false, error: String(error) });
    }
    setLoading(false);
  };

  const handleTestSearch = async () => {
    setLoading(true);
    try {
      const result = await searchMemory('test query', 5);
      setTestResult(result);
    } catch (error) {
      setTestResult({ success: false, error: String(error) });
    }
    setLoading(false);
  };

  const handleTestStore = async () => {
    setLoading(true);
    try {
      const result = await storeConversation(
        'test-conv-123',
        'Hello from user',
        'Hello from AI',
        { test: 'true' },
      );
      setTestResult(result);
    } catch (error) {
      setTestResult({ success: false, error: String(error) });
    }
    setLoading(false);
  };

  return (
    <div className="container mx-auto p-6 max-w-4xl">
      <h1 className="text-3xl font-bold mb-6">Python Plugin Test</h1>

      <div className="space-y-4">
        {/* Status Display */}
        {status && (
          <div className="p-4 bg-gray-100 dark:bg-gray-800 rounded-lg">
            <h2 className="text-xl font-semibold mb-2">Memory Status</h2>
            <div className="space-y-1 text-sm">
              <div>
                <strong>Initialized:</strong>{' '}
                <Badge variant={status.initialized ? 'default' : 'secondary'}>
                  {status.initialized ? 'Yes' : 'No'}
                </Badge>
              </div>
              <div>
                <strong>Available:</strong>{' '}
                <Badge variant={status.available ? 'default' : 'destructive'}>
                  {status.available ? 'Yes' : 'No'}
                </Badge>
              </div>
              <div>


              {status.kb_path && (
                <div>
                  <strong>KB Path:</strong> {status.kb_path}
                </div>
              )}
              {status.python_version && (
                <div>
                  <strong>Python:</strong> {status.python_version}
                </div>
              )}

            </div>
          </div>
        )}

        {/* Test Result Display */}
        {testResult && (
          <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <h2 className="text-xl font-semibold mb-2">Test Result</h2>
            <pre className="text-xs overflow-auto">
              {JSON.stringify(testResult, null, 2)}
            </pre>
          </div>
        )}

        {/* Test Buttons */}
        <div className="grid grid-cols-2 gap-4">
          <Button onClick={handleTestConnection} disabled={loading}>
            Test Connection
          </Button>

          <Button onClick={handleGetStatus} disabled={loading}>
            Get Status
          </Button>

          <Button onClick={handleInitialize} disabled={loading}>
            Initialize Memory
          </Button>

          <Button onClick={handleTestSearch} disabled={loading}>
            Test Search
          </Button>

          <Button onClick={handleTestStore} disabled={loading}>
            Test Store
          </Button>

          <Button
            onClick={() => {
              setStatus(null);
              setTestResult(null);
            }}
            variant="outline"
            disabled={loading}
          >
            Clear Results
          </Button>
        </div>

        {loading && (
          <div className="text-center text-gray-500">
            <div className="animate-spin inline-block w-6 h-6 border-2 border-current border-t-transparent rounded-full" />
            <p className="mt-2">Testing...</p>
          </div>
        )}
      </div>

      {/* Instructions */}
      <div className="mt-8 p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
        <h3 className="font-semibold mb-2">Instructions</h3>
        <ol className="list-decimal list-inside space-y-1 text-sm">
          <li>Click "Test Connection" to verify Python plugin works</li>
          <li>Click "Get Status" to check memory service status</li>
          <li>Click "Initialize Memory" to set up the knowledge base</li>
          <li>Click "Test Search" and "Test Store" to test operations</li>
        </ol>
      </div>
    </div>
  );
}
