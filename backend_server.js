const express = require('express');
const app = express();
const port = process.argv[2] || 3000;

app.get('/api/users', (req, res) => {
    res.json({
        server: `Backend on port ${port}`,
        users: [
            { id: 1, name: 'Alice' },
            { id: 2, name: 'Bob' }
        ]
    });
});

app.get('/api/health', (req, res) => {
    res.json({ status: 'healthy', port });
});

app.get('/', (req, res) => {
    res.send(`
        <h1>Backend Server on port ${port}</h1>
        <h2>Request Information:</h2>
        <pre>${JSON.stringify({
            headers: req.headers,
            url: req.url,
            method: req.method
        }, null, 2)}</pre>
    `);
});

app.listen(port, () => {
    console.log(`Backend server running on port ${port}`);
}); 
