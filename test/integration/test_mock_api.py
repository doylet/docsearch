#!/usr/bin/env python3
"""
Simple mock API server for testing Phase 2 CLI integration
"""
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse

class MockAPIHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        path = urlparse(self.path).path
        
        # Mock responses for different endpoints
        if path == '/api/status':
            response = {
                "status": "healthy",
                "collection": {
                    "name": "zero_latency_docs",
                    "documents": 42,
                    "chunks": 156,
                    "vector_dimensions": 384,
                    "last_updated": "2025-01-20T02:00:00Z"
                },
                "configuration": {
                    "embedding_model": "gte-small",
                    "vector_database": "qdrant",
                    "collection_name": "zero_latency_docs"
                },
                "performance": {
                    "avg_search_time_ms": 45.7,
                    "total_searches": 128,
                    "uptime_seconds": 3600
                }
            }
        elif path == '/api/docs':
            response = {
                "documents": [
                    {
                        "id": "doc_1",
                        "title": "Phase 2 Plan",
                        "path": "docs/misc/artefacts/022_phase-2-cli-interface-plan.md",
                        "indexed_at": "2025-01-20T01:30:00Z",
                        "chunk_count": 8
                    },
                    {
                        "id": "doc_2", 
                        "title": "API Design",
                        "path": "api/public/openapi.yaml",
                        "indexed_at": "2025-01-20T01:25:00Z",
                        "chunk_count": 12
                    }
                ],
                "total_count": 2
            }
        elif path.startswith('/api/search'):
            response = {
                "query": "API endpoints",
                "total_results": 3,
                "results": [
                    {
                        "score": 0.92,
                        "document_title": "API Design",
                        "content": "The API provides endpoints for searching, indexing, and managing documents...",
                        "snippet": "API provides endpoints for searching, indexing",
                        "section": "Overview",
                        "doc_type": "yaml"
                    }
                ],
                "search_metadata": {
                    "embedding_time_ms": 15,
                    "search_time_ms": 30,
                    "total_time_ms": 45,
                    "model_used": "gte-small"
                }
            }
        else:
            self.send_response(404)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({"error": "Not found"}).encode())
            return
            
        # Send successful response
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(response, indent=2).encode())
    
    def do_POST(self):
        path = urlparse(self.path).path
        
        if path == '/api/reindex':
            response = {
                "status": "success",
                "message": "Reindex completed successfully",
                "processed_documents": 42,
                "total_chunks": 156,
                "duration_seconds": 45.2
            }
        else:
            self.send_response(404)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({"error": "Not found"}).encode())
            return
            
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(response, indent=2).encode())
    
    def do_DELETE(self):
        path = urlparse(self.path).path
        
        if path.startswith('/api/docs/'):
            doc_id = path.split('/')[-1]
            response = {
                "status": "success",
                "message": f"Document {doc_id} deleted successfully"
            }
        else:
            self.send_response(404)
            self.send_header('Content-Type', 'application/json') 
            self.end_headers()
            self.wfile.write(json.dumps({"error": "Not found"}).encode())
            return
            
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(response, indent=2).encode())

    def log_message(self, format, *args):
        print(f"[MOCK API] {format % args}")

if __name__ == '__main__':
    server = HTTPServer(('localhost', 8081), MockAPIHandler)
    print("🧪 Mock API server running on http://localhost:8081")
    print("Available endpoints:")
    print("  GET  /api/status")
    print("  GET  /api/docs") 
    print("  GET  /api/search?q=...")
    print("  POST /api/reindex")
    print("  DELETE /api/docs/{id}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Mock server stopped")
