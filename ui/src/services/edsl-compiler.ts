import { EDSLCompilationResult, EDSLValidationResult, CompilerOptions } from '../types/edsl';

// Production service that communicates with Rust backend
export class EDSLCompilerService {
  private baseUrl: string;
  private websocket: WebSocket | null = null;
  private requestId = 0;
  private pendingRequests = new Map<string, {
    resolve: (value: any) => void;
    reject: (reason: any) => void;
  }>();

  constructor(baseUrl: string = 'http://localhost:3002') {
    this.baseUrl = baseUrl;
  }

  // HTTP API methods
  async compile(edslContent: string, options: CompilerOptions): Promise<EDSLCompilationResult> {
    console.log('📤 Compiling EDSL:', { 
      contentLength: edslContent.length,
      preview: edslContent.substring(0, 100) + (edslContent.length > 100 ? '...' : ''),
      options 
    });
    
    try {
      const response = await fetch(`${this.baseUrl}/api/compile`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          edsl_content: edslContent,
          layout: options.layout,
          verbose: options.verbose,
        }),
      });

      if (!response.ok) {
        // Try to get error message from response
        let errorMessage = 'Server error';
        try {
          const errorData = await response.json();
          errorMessage = errorData.error || errorMessage;
        } catch (e) {
          // Ignore JSON parsing errors
        }
        
        console.error('❌ Server compilation failed:', {
          status: response.status,
          statusText: response.statusText,
          error: errorMessage
        });
        
        // Don't fallback to mock for 4xx errors - show the actual error
        if (response.status >= 400 && response.status < 500) {
          return {
            success: false,
            error: errorMessage,
          };
        }
        
        // Fallback to mock compilation for 5xx errors
        console.warn('Server error, using mock compilation');
        return this.mockCompile(edslContent, options);
      }

      const result = await response.json();
      console.log('✅ Compilation result:', {
        success: result.success,
        elementCount: result.data ? (Array.isArray(result.data) ? result.data.length : 1) : 0,
        error: result.error
      });
      
      return {
        success: result.success,
        data: result.data,
        error: result.error,
      };
    } catch (error) {
      // Fallback to mock compilation on network error
      console.error('🔌 Network error, using mock compilation:', error);
      return this.mockCompile(edslContent, options);
    }
  }

  async validate(edslContent: string): Promise<EDSLValidationResult> {
    try {
      const response = await fetch(`${this.baseUrl}/api/validate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          edsl_content: edslContent,
        }),
      });

      if (!response.ok) {
        // Fallback to mock validation
        return this.mockValidate(edslContent);
      }

      const result = await response.json();
      return {
        isValid: result.is_valid,
        error: result.error,
      };
    } catch (error) {
      // Fallback to mock validation on network error
      return this.mockValidate(edslContent);
    }
  }

  // WebSocket API methods for real-time compilation
  async connectWebSocket(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        const wsUrl = this.baseUrl.replace('http://', 'ws://').replace('https://', 'wss://');
        this.websocket = new WebSocket(`${wsUrl}/api/ws`);

        // Set a timeout for connection
        const connectionTimeout = setTimeout(() => {
          if (this.websocket && this.websocket.readyState === WebSocket.CONNECTING) {
            this.websocket.close();
            reject(new Error('WebSocket connection timeout'));
          }
        }, 5000);

        this.websocket.onopen = () => {
          clearTimeout(connectionTimeout);
          console.log('WebSocket connected to EDSL server');
          resolve();
        };

        this.websocket.onerror = (error) => {
          clearTimeout(connectionTimeout);
          console.warn('WebSocket connection error:', error);
          reject(new Error('WebSocket connection failed'));
        };

        this.websocket.onmessage = (event) => {
          try {
            const message = JSON.parse(event.data);
            this.handleWebSocketMessage(message);
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
          }
        };

        this.websocket.onclose = (event) => {
          clearTimeout(connectionTimeout);
          if (event.wasClean) {
            console.log('WebSocket disconnected cleanly');
          } else {
            console.warn('WebSocket disconnected unexpectedly');
          }
          this.websocket = null;
          // Reject any pending requests
          this.pendingRequests.forEach(({ reject }) => {
            reject(new Error('WebSocket disconnected'));
          });
          this.pendingRequests.clear();
        };
      } catch (error) {
        reject(new Error(`Failed to create WebSocket: ${error instanceof Error ? error.message : 'Unknown error'}`));
      }
    });
  }

  private handleWebSocketMessage(message: any): void {
    const { type, id } = message;
    console.log('📥 WebSocket message received:', { type, id, success: message.success });
    
    const pending = this.pendingRequests.get(id);

    if (!pending) {
      console.warn('⚠️ No pending request for message:', id);
      return;
    }

    this.pendingRequests.delete(id);

    switch (type) {
      case 'compile_result':
        console.log('✅ WebSocket compile result:', {
          success: message.success,
          elementCount: message.data ? (Array.isArray(message.data) ? message.data.length : 1) : 0,
          error: message.error,
          duration_ms: message.duration_ms
        });
        pending.resolve({
          success: message.success,
          data: message.data,
          error: message.error,
        });
        break;
      case 'validate_result':
        console.log('✅ WebSocket validate result:', {
          isValid: message.is_valid,
          error: message.error
        });
        pending.resolve({
          isValid: message.is_valid,
          error: message.error,
        });
        break;
      case 'error':
        console.error('❌ WebSocket error:', message.message);
        pending.reject(new Error(message.message));
        break;
      default:
        console.log('ℹ️ WebSocket message:', message);
    }
  }

  async compileWebSocket(edslContent: string, options: CompilerOptions): Promise<EDSLCompilationResult> {
    if (!this.websocket || this.websocket.readyState !== WebSocket.OPEN) {
      console.log('🔄 WebSocket not available, falling back to HTTP');
      // Fallback to HTTP API
      return this.compile(edslContent, options);
    }

    console.log('🌐 Compiling via WebSocket:', {
      contentLength: edslContent.length,
      preview: edslContent.substring(0, 100) + (edslContent.length > 100 ? '...' : '')
    });

    return new Promise((resolve, reject) => {
      const id = `compile_${++this.requestId}`;
      this.pendingRequests.set(id, { resolve, reject });

      const message = {
        type: 'compile',
        id,
        edsl_content: edslContent,
        layout: options.layout,
        verbose: options.verbose,
      };
      
      console.log('📤 Sending WebSocket message:', { id, type: 'compile' });
      this.websocket!.send(JSON.stringify(message));

      // Timeout after 10 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          console.error('⏱️ WebSocket request timeout:', id);
          this.pendingRequests.delete(id);
          reject(new Error('WebSocket request timeout'));
        }
      }, 10000);
    });
  }

  async validateWebSocket(edslContent: string): Promise<EDSLValidationResult> {
    if (!this.websocket || this.websocket.readyState !== WebSocket.OPEN) {
      // Fallback to HTTP API
      return this.validate(edslContent);
    }

    return new Promise((resolve, reject) => {
      const id = `validate_${++this.requestId}`;
      this.pendingRequests.set(id, { resolve, reject });

      this.websocket!.send(JSON.stringify({
        type: 'validate',
        id,
        edsl_content: edslContent,
      }));

      // Timeout after 5 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error('WebSocket request timeout'));
        }
      }, 5000);
    });
  }

  disconnectWebSocket(): void {
    if (this.websocket) {
      try {
        if (this.websocket.readyState === WebSocket.OPEN || this.websocket.readyState === WebSocket.CONNECTING) {
          this.websocket.close();
        }
      } catch (error) {
        console.warn('Error closing WebSocket:', error);
      } finally {
        this.websocket = null;
        // Clear any pending requests
        this.pendingRequests.forEach(({ reject }) => {
          reject(new Error('WebSocket manually disconnected'));
        });
        this.pendingRequests.clear();
      }
    }
  }

  // Health check
  async healthCheck(): Promise<{ status: string; version: string; features: string[] }> {
    try {
      const response = await fetch(`${this.baseUrl}/health`);
      return await response.json();
    } catch (error) {
      throw new Error(`Health check failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  // Mock methods for fallback when backend is unavailable
  private async mockCompile(edslContent: string, _options: CompilerOptions): Promise<EDSLCompilationResult> {
    try {
      // Simple mock compilation
      const elements = this.createMockExcalidrawElements(edslContent);
      return {
        success: true,
        data: elements,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Mock compilation failed',
      };
    }
  }

  private async mockValidate(edslContent: string): Promise<EDSLValidationResult> {
    try {
      // Basic validation - check for syntax issues
      const lines = edslContent.split('\n');
      let inConfig = false;
      
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();
        
        if (line === '---') {
          inConfig = !inConfig;
          continue;
        }
        
        if (inConfig || line === '' || line.startsWith('#')) {
          continue;
        }
        
        // Check for unclosed style blocks
        if (line.includes('{') && !line.includes('}')) {
          let found = false;
          for (let j = i + 1; j < lines.length; j++) {
            if (lines[j].includes('}')) {
              found = true;
              break;
            }
          }
          if (!found) {
            return {
              isValid: false,
              error: `Unclosed style block at line ${i + 1}`,
            };
          }
        }
      }
      
      return { isValid: true };
    } catch (error) {
      return {
        isValid: false,
        error: error instanceof Error ? error.message : 'Mock validation failed',
      };
    }
  }

  private createMockExcalidrawElements(edslContent: string): any[] {
    const elements: any[] = [];
    const lines = edslContent.split('\n');
    const nodes: { [key: string]: { label: string; x: number; y: number } } = {};
    let nodeCount = 0;
    
    // Extract nodes
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed === '' || trimmed.startsWith('#') || trimmed.startsWith('---') || trimmed.includes('->')) {
        continue;
      }
      
      // Simple node parsing: node_id[Label] or node_id
      const nodeMatch = trimmed.match(/^(\w+)(?:\[([^\]]+)\])?/);
      if (nodeMatch) {
        const [, id, label] = nodeMatch;
        if (!nodes[id]) {
          nodes[id] = {
            label: label || id,
            x: (nodeCount % 3) * 200 + 100,
            y: Math.floor(nodeCount / 3) * 150 + 100,
          };
          nodeCount++;
        }
      }
    }
    
    // Create node elements
    Object.entries(nodes).forEach(([id, node]) => {
      elements.push({
        type: 'rectangle',
        id: `node_${id}`,
        x: node.x - 60,
        y: node.y - 30,
        width: 120,
        height: 60,
        angle: 0,
        strokeColor: '#000000',
        backgroundColor: 'transparent',
        fillStyle: 'solid',
        strokeWidth: 2,
        strokeStyle: 'solid',
        roughness: 1,
        opacity: 100,
        text: node.label,
        fontSize: 16,
        fontFamily: 1,
        textAlign: 'center',
        verticalAlign: 'middle',
      });
    });
    
    // Extract and create edges
    let edgeCount = 0;
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed.includes('->')) {
        const parts = trimmed.split('->');
        for (let i = 0; i < parts.length - 1; i++) {
          const fromId = parts[i].trim().replace(/\[.*\]/, '');
          const toId = parts[i + 1].trim().replace(/\[.*\]/, '').replace(/:.*/, '');
          
          if (nodes[fromId] && nodes[toId]) {
            const from = nodes[fromId];
            const to = nodes[toId];
            
            elements.push({
              type: 'arrow',
              id: `edge_${edgeCount++}`,
              x: from.x,
              y: from.y,
              width: to.x - from.x,
              height: to.y - from.y,
              angle: 0,
              strokeColor: '#000000',
              backgroundColor: 'transparent',
              fillStyle: 'solid',
              strokeWidth: 2,
              strokeStyle: 'solid',
              roughness: 1,
              opacity: 100,
              points: [[0, 0], [to.x - from.x, to.y - from.y]],
              lastCommittedPoint: [to.x - from.x, to.y - from.y],
              startBinding: {
                elementId: `node_${fromId}`,
                focus: 0,
                gap: 0,
              },
              endBinding: {
                elementId: `node_${toId}`,
                focus: 0,
                gap: 0,
              },
              endArrowhead: 'triangle',
            });
          }
        }
      }
    }
    
    return elements;
  }
}
