import { createApp } from 'vue'
import App           from './App.vue'
import './style.css'

// ── Bloquer tous les raccourcis clavier qui pourraient
//    quitter le mode plein écran ou fermer l'application ──
window.addEventListener('keydown', (e: KeyboardEvent) => {
    const blocked = [
        e.key === 'F11',                           // Basculer plein écran
        e.key === 'F4'  && e.altKey,               // Alt+F4  — fermer
        e.key === 'F4'  && e.metaKey,              // Cmd+F4
        e.key === 'w'   && (e.ctrlKey || e.metaKey), // Ctrl/Cmd+W — fermer onglet
        e.key === 'F5'  && e.ctrlKey,              // Ctrl+F5 — rechargement
        e.key === 'r'   && (e.ctrlKey || e.metaKey), // Ctrl/Cmd+R — recharger
        e.key === 'F5',                            // F5 — recharger
        e.key === 'Escape',                        // Échap — sortir du plein écran
        e.key === 'Tab' && e.altKey,               // Alt+Tab — changer fenêtre
        e.key === 'F1',                            // Aide Windows
        e.key === 'F3',                            // Recherche navigateur
        e.key === 'F6',                            // Barre d'adresse
        e.key === 'F10',                           // Menu navigateur
        e.key === 'F12',                           // DevTools
        e.key === 'i' && e.ctrlKey && e.shiftKey, // Ctrl+Shift+I — DevTools
        e.key === 'j' && e.ctrlKey && e.shiftKey, // Ctrl+Shift+J — Console
        e.key === 'u' && e.ctrlKey,               // Ctrl+U — source
        e.key === 'p' && e.ctrlKey,               // Ctrl+P — imprimer navigateur
    ]

    if (blocked.some(Boolean)) {
        e.preventDefault()
        e.stopPropagation()
        return false
    }
}, { capture: true })

// Bloquer le menu contextuel (clic droit)
window.addEventListener('contextmenu', (e) => {
    e.preventDefault()
}, { capture: true })

// Bloquer la sélection de texte globale (sauf champs input)
window.addEventListener('selectstart', (e) => {
    if (!(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement)) {
        e.preventDefault()
    }
}, { capture: true })

createApp(App).mount('#app')