var module = module || {
  exports: async () => {
    const { value } = await Wapo.run(`
      var module = {
        exports: () => {
          const value = new Map()
          value.set('foo', 'bar')
          console.log(typeof value)
          console.log(value)
          console.log(Object.prototype.toString.call(value))
          return value
        }
      }
    `)
    console.log(typeof value)
    console.log(value)
    console.log(Object.prototype.toString.call(value))
  }
};
