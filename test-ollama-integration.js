#!/usr/bin/env node

/**
 * Test script to verify Ollama integration with n8n workflows
 * Run this after setting up Ollama and n8n
 */

const http = require('http');

const OLLAMA_URL = 'http://localhost:11434';
const N8N_URL = 'http://localhost:5678';

async function testOllamaConnection() {
    console.log('🔍 Testing Ollama connection...');
    
    try {
        const response = await fetch(`${OLLAMA_URL}/api/tags`);
        if (response.ok) {
            const data = await response.json();
            console.log('✅ Ollama is running');
            console.log('📦 Installed models:', data.models?.map(m => m.name).join(', ') || 'None');
            return true;
        }
    } catch (error) {
        console.log('❌ Ollama is not running or not accessible');
        console.log('💡 Run: ollama serve');
        return false;
    }
}

async function testN8nConnection() {
    console.log('🔍 Testing n8n connection...');
    
    try {
        const response = await fetch(`${N8N_URL}/api/v1/health`);
        if (response.ok) {
            console.log('✅ n8n is running');
            return true;
        }
    } catch (error) {
        console.log('❌ n8n is not running or not accessible');
        console.log('💡 Run: npm run n8n:start');
        return false;
    }
}

async function testOllamaModel() {
    console.log('🔍 Testing Ollama model (llama3.2:3b)...');
    
    try {
        const response = await fetch(`${OLLAMA_URL}/api/generate`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                model: 'llama3.2:3b',
                prompt: 'Hello, are you working?',
                stream: false
            })
        });
        
        if (response.ok) {
            const data = await response.json();
            console.log('✅ Ollama model is responding');
            console.log('🤖 Response:', data.response?.substring(0, 100) + '...');
            return true;
        }
    } catch (error) {
        console.log('❌ Ollama model test failed');
        console.log('💡 Run: ollama pull llama3.2:3b');
        return false;
    }
}

async function testN8nWorkflow() {
    console.log('🔍 Testing n8n AI workflow...');
    
    try {
        const testData = {
            path: '/tmp/test-project',
            framework: 'Node.js',
            package_manager: 'npm'
        };
        
        const response = await fetch(`${N8N_URL}/webhook/ai-suggestions`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(testData)
        });
        
        if (response.ok) {
            const data = await response.json();
            console.log('✅ n8n AI workflow is working');
            console.log('📊 Response:', data.message || 'Success');
            return true;
        }
    } catch (error) {
        console.log('❌ n8n AI workflow test failed');
        console.log('💡 Make sure AI workflows are imported in n8n');
        return false;
    }
}

async function main() {
    console.log('🚀 Testing Ollama + n8n Integration\n');
    
    const ollamaOk = await testOllamaConnection();
    const n8nOk = await testN8nConnection();
    
    if (ollamaOk && n8nOk) {
        await testOllamaModel();
        await testN8nWorkflow();
    }
    
    console.log('\n📋 Setup Checklist:');
    console.log('1. Install Ollama: curl -fsSL https://ollama.ai/install.sh | sh');
    console.log('2. Start Ollama: ollama serve');
    console.log('3. Pull models: npm run ollama:models');
    console.log('4. Start n8n: npm run n8n:start');
    console.log('5. Import AI workflows in n8n UI');
    console.log('\n🎯 Access URLs:');
    console.log('- n8n UI: http://localhost:5678 (admin/portal123)');
    console.log('- Ollama API: http://localhost:11434');
}

main().catch(console.error);
