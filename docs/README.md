# Beautibuk Agent

An AI-powered booking agent for finding and booking beauty salons. Built with Rust for high performance and reliability.

## Overview

This system provides a chat-based interface where users can interact with an AI agent to find nearby beauty salons and make bookings. The agent understands natural language requests, searches for salons based on location, checks availability, and facilitates the booking process.

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed system architecture, component design, and technology stack decisions.

## Documentation

- **[SUMMARY.md](./SUMMARY.md)** - Quick overview and key decisions
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - System architecture and design
- **[MCP_INTEGRATION.md](./MCP_INTEGRATION.md)** - MCP integration analysis and benefits ⭐
- **[MCP_CLIENT_IMPLEMENTATION.md](./MCP_CLIENT_IMPLEMENTATION.md)** - MCP client implementation guide ⭐
- **[IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md)** - Step-by-step implementation guide
- **[API_INTEGRATION.md](./API_INTEGRATION.md)** - Python backend REST API details (alternative to MCP)
- **[VECTOR_DATABASE.md](./VECTOR_DATABASE.md)** - Vector database setup and usage
- **[VECTOR_DB_BENEFITS.md](./VECTOR_DB_BENEFITS.md)** - Benefits and business data storage strategies
- **[TESTING_LLM_OPTIONS.md](./TESTING_LLM_OPTIONS.md)** - Free LLM testing options (Groq, Google AI Studio)
- **[COMMUNICATION_FLOW.md](./COMMUNICATION_FLOW.md)** - Communication flow diagrams
- **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)** - Code snippets and patterns

## Quick Start

1. **Prerequisites**
   - Rust installed (latest stable)
   - Python backend (`beautibuk-back`) running on `http://localhost:8000`
   - PostgreSQL with pgvector extension
   - **MCP Server**: Python backend MCP server running (port 8002)
   - **LLM Providers**: 
     - **Groq API key** (fast, free tier, function calling) - Get from https://console.groq.com
     - **Google AI Studio API key** (quality, embeddings, function calling) - Get from https://aistudio.google.com
     - See `TESTING_LLM_OPTIONS.md` for detailed setup
   - See `MCP_INTEGRATION.md` for MCP benefits and architecture

2. **Setup Rust Project**
   ```bash
   cargo new beautibuk-agent
   cd beautibuk-agent
   ```

3. **Review Architecture**
   - Read `ARCHITECTURE.md` for system design
   - Understand integration with Python backend
   - Read `API_INTEGRATION.md` for API details

4. **Follow Implementation Guide**
   - Start with Phase 1: Project Setup
   - Implement MCP client first (Phase 2) - see `MCP_CLIENT_IMPLEMENTATION.md`
   - Progress through each phase systematically
   - Focus on MVP features first

## Key Features

- 🤖 **AI Agent**: Natural language understanding for booking requests
- 🔧 **MCP Integration**: LLM autonomously calls backend tools via Model Context Protocol
- 🔍 **Semantic Search**: Vector database for intelligent business matching
- 📍 **Location-Based Search**: Find salons by proximity
- 📅 **Availability Checking**: Real-time availability verification
- 💬 **Chat Interface**: Intuitive conversation-based booking
- 🧠 **RAG (Retrieval Augmented Generation)**: Learn from past conversations
- ⚡ **High Performance**: Rust-based backend for speed and reliability
- 🎯 **Simplified Architecture**: 70% less code with MCP - LLM handles tool selection

## Technology Stack

- **Backend**: Rust (Axum)
- **Vector Database**: PostgreSQL with pgvector extension
- **Python Backend**: FastAPI (existing - beautibuk-back)
- **LLM**: OpenAI/Anthropic API or Ollama (local)
- **Frontend**: React/Vue.js or vanilla JavaScript

## Project Status

🚧 **In Development** - This is a new project. Follow the implementation guide to build the system step by step.
