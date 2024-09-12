var module = module || {
  exports: async () => {
    const { value } = await Wapo.run(`
      var module = {
        exports: () => {
          const value = null
          console.log(typeof value)
          console.log(value)
          return value
        }
      }
    `)
    console.log(typeof value)
    console.log(value)
  }
};
