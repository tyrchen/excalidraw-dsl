import React, { useCallback, useEffect, useRef, useState } from 'react';
import Editor from '@monaco-editor/react';
import { useEDSLStore } from '../store/edsl-store';
import { EDSLCompilerService } from '../services/edsl-compiler';
import debounce from 'lodash.debounce';
import { AlertCircle, CheckCircle, Loader2, Wifi, WifiOff } from 'lucide-react';
import { Alert, AlertDescription } from './ui/alert';

const compilerService = new EDSLCompilerService();

interface EdslEditorProps {
  height?: string;
}

export const EdslEditor: React.FC<EdslEditorProps> = ({ height = '100%' }) => {
  const {
    editorContent,
    setEditorContent,
    compilerOptions,
    validationResult,
    isValidating,
    setIsValidating,
    setValidationResult,
    setCompilationResult,
    setIsCompiling,
    currentFile,
    updateFileContent,
  } = useEDSLStore();

  const editorRef = useRef<any>(null);
  const [isWebSocketConnected, setIsWebSocketConnected] = useState(false);
  const [useWebSocket, setUseWebSocket] = useState(true);

  // Initialize WebSocket connection
  useEffect(() => {
    if (useWebSocket) {
      compilerService.connectWebSocket()
        .then(() => {
          setIsWebSocketConnected(true);
          console.log('WebSocket connected for real-time compilation');
        })
        .catch((error) => {
          console.warn('WebSocket connection failed, falling back to HTTP:', error.message || error);
          setIsWebSocketConnected(false);
        });

      return () => {
        try {
          compilerService.disconnectWebSocket();
        } catch (error) {
          // Ignore disconnection errors
        }
        setIsWebSocketConnected(false);
      };
    }
  }, [useWebSocket]);

  // Debounced validation
  const debouncedValidate = useCallback(
    debounce(async (content: string) => {
      if (!compilerOptions.validate) return;
      
      setIsValidating(true);
      try {
        const result = isWebSocketConnected
          ? await compilerService.validateWebSocket(content)
          : await compilerService.validate(content);
        setValidationResult(result);
      } catch (error) {
        setValidationResult({
          isValid: false,
          error: error instanceof Error ? error.message : 'Validation failed',
        });
      } finally {
        setIsValidating(false);
      }
    }, 500),
    [compilerOptions.validate, setIsValidating, setValidationResult, isWebSocketConnected]
  );

  // Debounced compilation
  const debouncedCompile = useCallback(
    debounce(async (content: string) => {
      setIsCompiling(true);
      try {
        const result = isWebSocketConnected
          ? await compilerService.compileWebSocket(content, compilerOptions)
          : await compilerService.compile(content, compilerOptions);
        setCompilationResult(result);
      } catch (error) {
        setCompilationResult({
          success: false,
          error: error instanceof Error ? error.message : 'Compilation failed',
        });
      } finally {
        setIsCompiling(false);
      }
    }, 1000),
    [compilerOptions, setIsCompiling, setCompilationResult, isWebSocketConnected]
  );

  const handleEditorChange = useCallback((value: string | undefined) => {
    const content = value || '';
    setEditorContent(content);
    
    // Update current file if one is selected
    if (currentFile) {
      updateFileContent(currentFile.name, content);
    }
    
    // Trigger validation and compilation
    debouncedValidate(content);
    debouncedCompile(content);
  }, [setEditorContent, currentFile, updateFileContent, debouncedValidate, debouncedCompile]);

  const handleEditorDidMount = (editor: any) => {
    editorRef.current = editor;
    
    // Configure EDSL language features
    const monaco = (window as any).monaco;
    if (monaco) {
      // Register custom language for EDSL
      monaco.languages.register({ id: 'edsl' });
      
      // Define syntax highlighting
      monaco.languages.setMonarchTokensProvider('edsl', {
        tokenizer: {
          root: [
            // Comments
            [/#.*/, 'comment'],
            
            // YAML frontmatter
            [/---/, 'delimiter'],
            
            // Node definitions
            [/\w+\[.*?\]/, 'entity.name.function'],
            [/\w+/, 'identifier'],
            
            // Arrows
            [/->|<->|~>|--/, 'operator'],
            
            // Style blocks
            [/\{/, 'bracket.open'],
            [/\}/, 'bracket.close'],
            [/:/, 'delimiter'],
            [/;/, 'delimiter'],
            
            // Strings
            [/"[^"]*"/, 'string'],
            
            // Colors
            [/#[0-9a-fA-F]{6}/, 'number.hex'],
            
            // Numbers
            [/\d+(\.\d+)?/, 'number'],
            
            // Keywords
            [/\b(container|as|shape|strokeColor|backgroundColor|strokeWidth|roughness|layout|theme|font|fontSize)\b/, 'keyword'],
          ],
        },
      });
      
      // Define theme
      monaco.editor.defineTheme('edsl-theme', {
        base: 'vs',
        inherit: true,
        rules: [
          { token: 'comment', foreground: '6a9955' },
          { token: 'keyword', foreground: '0000ff' },
          { token: 'string', foreground: 'a31515' },
          { token: 'number', foreground: '098658' },
          { token: 'number.hex', foreground: '098658' },
          { token: 'entity.name.function', foreground: '795e26' },
          { token: 'operator', foreground: 'af00db' },
          { token: 'identifier', foreground: '001080' },
        ],
        colors: {},
      });
      
      monaco.editor.setTheme('edsl-theme');
    }
  };

  // Initial validation
  useEffect(() => {
    if (editorContent && compilerOptions.validate) {
      debouncedValidate(editorContent);
    }
  }, []);

  return (
    <div className="flex flex-col h-full">
      {/* Connection & Validation Status */}
      <div className="border-b">
        {/* Connection Status */}
        <div className="px-2 py-1 bg-gray-50 border-b flex items-center justify-between text-xs">
          <div className="flex items-center space-x-2">
            {isWebSocketConnected ? (
              <>
                <Wifi className="h-3 w-3 text-green-600" />
                <span className="text-green-700">WebSocket connected (real-time)</span>
              </>
            ) : useWebSocket ? (
              <>
                <WifiOff className="h-3 w-3 text-orange-600" />
                <span className="text-orange-700">Server offline - using mock compiler</span>
              </>
            ) : (
              <>
                <WifiOff className="h-3 w-3 text-gray-600" />
                <span className="text-gray-700">WebSocket disabled - using HTTP</span>
              </>
            )}
          </div>
          <button
            onClick={() => setUseWebSocket(!useWebSocket)}
            className="text-blue-600 hover:text-blue-800 underline"
          >
            {useWebSocket ? 'Disable' : 'Enable'} WebSocket
          </button>
        </div>

        {/* Validation Status */}
        {(validationResult || isValidating) && (
          <div className="p-2">
            {isValidating ? (
              <Alert>
                <Loader2 className="h-4 w-4 animate-spin" />
                <AlertDescription>Validating...</AlertDescription>
              </Alert>
            ) : validationResult?.isValid ? (
              <Alert className="border-green-200 bg-green-50">
                <CheckCircle className="h-4 w-4 text-green-600" />
                <AlertDescription className="text-green-800">
                  EDSL syntax is valid
                </AlertDescription>
              </Alert>
            ) : (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {validationResult?.error || 'Syntax error'}
                </AlertDescription>
              </Alert>
            )}
          </div>
        )}
      </div>
      
      {/* Editor */}
      <div className="flex-1">
        <Editor
          height={height}
          language="edsl"
          value={editorContent}
          onChange={handleEditorChange}
          onMount={handleEditorDidMount}
          options={{
            minimap: { enabled: false },
            lineNumbers: 'on',
            wordWrap: 'on',
            automaticLayout: true,
            fontSize: 14,
            fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
            tabSize: 2,
            insertSpaces: true,
            scrollBeyondLastLine: false,
            folding: true,
            lineDecorationsWidth: 10,
            lineNumbersMinChars: 3,
            renderLineHighlight: 'line',
            contextmenu: true,
            selectOnLineNumbers: true,
            bracketPairColorization: {
              enabled: true,
            },
          }}
        />
      </div>
    </div>
  );
};