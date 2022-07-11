PetiteVue.createApp({
  theme: localStorage.theme,
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
}).mount()
