// Test script to verify n8n integration
const { invoke } = require('@tauri-apps/api/core');

async function testN8nIntegration() {
    try {
        console.log('Testing n8n integration...');
        
        // Test 1: Check n8n health
        console.log('1. Checking n8n health...');
        const isHealthy = await invoke('check_n8n_health');
        console.log('n8n health:', isHealthy);
        
        // Test 2: List available workflows
        console.log('2. Listing available workflows...');
        const workflows = await invoke('list_available_workflows');
        console.log('Available workflows:', workflows);
        
        // Test 3: Get suggested workflows for a Node.js project
        console.log('3. Getting suggested workflows for Node.js...');
        const suggested = await invoke('get_suggested_workflows', {
            framework: 'Node.js',
            packageManager: 'npm'
        });
        console.log('Suggested workflows:', suggested);
        
        console.log('✅ n8n integration test completed successfully!');
        
    } catch (error) {
        console.error('❌ n8n integration test failed:', error);
    }
}

// Run the test
testN8nIntegration();
