import React, { useEffect, useState } from 'react';
import { Excalidraw } from '@excalidraw/excalidraw';
import '@excalidraw/excalidraw/index.css';
import { useEDSLStore } from '../store/edsl-store';
import { AlertCircle, Bug, ChevronDown, ChevronUp } from 'lucide-react';
import { Alert, AlertDescription } from './ui/alert';
import { Button } from './ui/button';

interface ExcalidrawPreviewProps {
  className?: string;
}

export const ExcalidrawPreview: React.FC<ExcalidrawPreviewProps> = ({ className }) => {
  const { compilationResult, isCompiling, editorContent } = useEDSLStore();
  const [elements, setElements] = useState<any[]>([]);
  const [appState, setAppState] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);
  const [showDebugPanel, setShowDebugPanel] = useState(false);
  const [debugData, setDebugData] = useState<any>(null);
  const [excalidrawKey, setExcalidrawKey] = useState(0);

  useEffect(() => {
    console.log('Compilation result received:', compilationResult);
    
    if (compilationResult) {
      // Store debug data
      setDebugData({
        compilationResult,
        edslContent: editorContent,
        timestamp: new Date().toISOString(),
      });

      if (compilationResult.success && compilationResult.data) {
        try {
          console.log('Raw compilation data:', compilationResult.data);
          
          // Check if this is the full Excalidraw file format
          let elementsToRender: any[] = [];
          
          if (compilationResult.data.type === 'excalidraw' && compilationResult.data.elements) {
            // New format: full Excalidraw file
            console.log('Detected full Excalidraw file format');
            elementsToRender = compilationResult.data.elements;
            // Also extract appState if available
            if (compilationResult.data.appState) {
              setAppState(compilationResult.data.appState);
            }
          } else if (Array.isArray(compilationResult.data)) {
            // Legacy format: array of elements
            console.log('Detected legacy array format');
            elementsToRender = compilationResult.data;
          } else {
            // Unknown format
            console.error('Unknown compilation data format:', compilationResult.data);
            throw new Error('Unknown compilation data format');
          }

          console.log(`Found ${elementsToRender.length} elements to render`);
          
          // The elements from the server should already be in the correct Excalidraw format
          // Just ensure they're valid
          const validElements = elementsToRender.filter(el => el && el.type && el.id);
          
          console.log('Valid Excalidraw elements:', validElements);
          setElements(validElements);
          setError(null);
          // Force re-render of Excalidraw by changing key
          setExcalidrawKey(prev => prev + 1);
        } catch (err) {
          const errorMsg = err instanceof Error ? err.message : 'Failed to render diagram';
          console.error('Error processing elements:', err);
          setError(errorMsg);
          setElements([]);
          setAppState(null);
        }
      } else {
        const errorMsg = compilationResult.error || 'Compilation failed';
        console.warn('Compilation failed:', errorMsg);
        setError(errorMsg);
        setElements([]);
        setAppState(null);
      }
    } else {
      console.log('No compilation result');
      setDebugData(null);
    }
  }, [compilationResult, editorContent]);

  if (error) {
    // Parse error messages to provide helpful hints
    const isParseError = error.includes('Parse error:');
    const isColorError = error.includes('backgroundColor') || error.includes('strokeColor');
    
    return (
      <div className={`p-4 ${className} space-y-3`}>
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            <div className="space-y-2">
              <div className="font-semibold">Compilation Error</div>
              <pre className="text-xs whitespace-pre-wrap bg-red-50 dark:bg-red-950 p-2 rounded">
                {error}
              </pre>
              {isParseError && isColorError && (
                <div className="mt-2 text-xs">
                  <strong>💡 Hint:</strong> Colors in style blocks must be quoted strings.
                  <br />
                  Use <code className="bg-gray-100 dark:bg-gray-800 px-1 rounded">"#3b82f6"</code> instead of <code className="bg-gray-100 dark:bg-gray-800 px-1 rounded">#3b82f6</code>
                </div>
              )}
              {isParseError && !isColorError && (
                <div className="mt-2 text-xs">
                  <strong>💡 Hint:</strong> Check your syntax. Make sure all nodes are defined before being referenced in edges.
                </div>
              )}
            </div>
          </AlertDescription>
        </Alert>
        
        {/* Show debug panel automatically on error */}
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowDebugPanel(!showDebugPanel)}
          className="w-full"
        >
          <Bug className="h-3 w-3 mr-1" />
          {showDebugPanel ? 'Hide' : 'Show'} Debug Information
        </Button>
        
        {showDebugPanel && debugData && (
          <div className="bg-gray-900 text-green-400 p-3 text-xs font-mono rounded overflow-y-auto max-h-64">
            <div className="mb-2">
              <strong className="text-yellow-400">Debug Information:</strong>
            </div>
            <div className="space-y-2">
              <div>
                <span className="text-blue-400">Error Type:</span> {debugData.compilationResult.success ? 'None' : 'Compilation Failed'}
              </div>
              <div>
                <span className="text-blue-400">Raw EDSL Input:</span>
                <pre className="mt-1 text-xs bg-gray-800 p-2 rounded overflow-x-auto">
                  {debugData.edslContent || 'No content available'}
                </pre>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  }

  // Show placeholder when no elements
  if (elements.length === 0 && !isCompiling) {
    return (
      <div className={`h-full ${className} flex items-center justify-center bg-gray-50`}>
        <div className="text-center text-gray-500">
          <div className="text-lg mb-2">No diagram to display</div>
          <div className="text-sm">Edit EDSL content to see the preview</div>
        </div>
      </div>
    );
  }

  return (
    <div className={`h-full ${className} flex flex-col`} style={{ minHeight: '400px' }}>
      {/* Debug Panel Toggle */}
      <div className="flex-shrink-0 border-b bg-gray-50 px-3 py-2 flex items-center justify-between">
        <span className="text-sm font-medium text-gray-700">
          Preview ({elements.length} elements)
        </span>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setShowDebugPanel(!showDebugPanel)}
          className="h-6 text-xs"
        >
          <Bug className="h-3 w-3 mr-1" />
          Debug
          {showDebugPanel ? <ChevronUp className="h-3 w-3 ml-1" /> : <ChevronDown className="h-3 w-3 ml-1" />}
        </Button>
      </div>

      {/* Debug Panel */}
      {showDebugPanel && (
        <div className="flex-shrink-0 bg-gray-900 text-green-400 p-3 text-xs font-mono max-h-64 overflow-y-auto border-b">
          <div className="mb-2">
            <strong className="text-yellow-400">Debug Information:</strong>
          </div>
          {debugData ? (
            <div className="space-y-2">
              <div>
                <span className="text-blue-400">Timestamp:</span> {debugData.timestamp}
              </div>
              <div>
                <span className="text-blue-400">Compilation Success:</span> {debugData.compilationResult.success ? '✅' : '❌'}
              </div>
              {debugData.compilationResult.error && (
                <div>
                  <span className="text-red-400">Error:</span> {debugData.compilationResult.error}
                </div>
              )}
              <div>
                <span className="text-blue-400">Raw Data:</span>
                <pre className="mt-1 text-xs bg-gray-800 p-2 rounded overflow-x-auto">
                  {JSON.stringify(debugData.compilationResult.data, null, 2)}
                </pre>
              </div>
              <div>
                <span className="text-blue-400">Processed Elements ({elements.length}):</span>
                <pre className="mt-1 text-xs bg-gray-800 p-2 rounded overflow-x-auto">
                  {JSON.stringify(elements, null, 2)}
                </pre>
              </div>
            </div>
          ) : (
            <div className="text-gray-500">No compilation data available</div>
          )}
        </div>
      )}

      {/* Main Preview Area */}
      <div className="flex-1 relative" style={{ height: '100%', minHeight: '400px' }}>
        {elements.length > 0 ? (
          <Excalidraw
            // Use key to force re-mount when elements change
            key={excalidrawKey}
            initialData={{
              elements,
              appState: {
                viewBackgroundColor: appState?.viewBackgroundColor || '#ffffff',
                currentItemFontFamily: 3,  // Use Cascadia/Code font
                zoom: {
                  value: 1,  // Start at 100% zoom
                },
                scrollToContent: true,  // Auto-scroll to content
                ...(appState || {}),
              },
              scrollToContent: true,  // Ensure content is visible and centered
            }}
            UIOptions={{
              canvasActions: {
                changeViewBackgroundColor: true,
                clearCanvas: false,
                export: { saveFileToDisk: true },
                loadScene: false,
                saveToActiveFile: false,
                saveAsImage: true,
                theme: true,
              },
            }}
            viewModeEnabled={false}
            zenModeEnabled={false}
            gridModeEnabled={false}
          />
        ) : !isCompiling ? (
          <div className="h-full flex items-center justify-center bg-gray-50">
            <div className="text-center text-gray-500">
              <div className="text-lg mb-2">No diagram to display</div>
              <div className="text-sm">
                {debugData ? 
                  `Compilation ${debugData.compilationResult.success ? 'succeeded' : 'failed'} but no elements generated` :
                  'Edit EDSL content to see the preview'
                }
              </div>
              {debugData && !debugData.compilationResult.success && (
                <div className="text-red-500 text-sm mt-2">
                  Error: {debugData.compilationResult.error}
                </div>
              )}
            </div>
          </div>
        ) : null}
        
        {isCompiling && (
          <div className="absolute inset-0 bg-white/50 backdrop-blur-sm flex items-center justify-center">
            <div className="bg-white p-4 rounded-lg shadow-lg">
              <div className="flex items-center space-x-2">
                <div className="animate-spin h-4 w-4 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                <span>Compiling EDSL...</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};