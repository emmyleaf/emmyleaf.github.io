const lsTheme = localStorage.theme
const prefersDark = (!lsTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)
if (prefersDark || lsTheme === 'dark') {
    document.documentElement.classList.add('dark')
} else {
    document.documentElement.classList.remove('dark')
}

document.addEventListener('alpine:init', () => {
    Alpine.data('theming', () => ({
        theme: lsTheme,
        toggle() {
            if (this.theme === 'dark') {
                this.theme = 'light'
                document.documentElement.classList.remove('dark')
            } else {
                this.theme = 'dark'
                document.documentElement.classList.add('dark')
            }
            localStorage.theme = this.theme
        },
    }))
})
