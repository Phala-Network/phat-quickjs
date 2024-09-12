var module = module || {
  exports: async () => {
    {
      const { value } = await Wapo.run(`
        var module = {
          exports: () => {
            const value = true
            console.log(typeof value)
            console.log(value)
            return value
          }
        }
      `)
      console.log(typeof value)
      console.log(value)
    }

    {
      const { value } = await Wapo.run(`
        var module = {
          exports: () => {
            const value = false
            console.log(typeof value)
            console.log(value)
            return value
          }
        }
      `)
      console.log(typeof value)
      console.log(value)
    }
  }
};
