// index.js - Entry point for the Grafbase Pagination Demo Application
// This file simply imports the main application logic from app.js

import './app.js';

// Ensure the application is initialized when the DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  console.log('Grafbase Pagination Demo initialized');
  
  // Display a helpful message if the API URL needs to be updated
  if (window.location.hostname !== 'localhost' && !localStorage.getItem('api_notice_shown')) {
    console.info('If you encounter API connection issues, you may need to update the API_URL in app.js');
    localStorage.setItem('api_notice_shown', 'true');
  }
});