var module = module || { exports: {} };
module.exports = async function main() {
  const ret = await Wapo.run(`
    async function some_async_func() {
      throw new Error('some error')
    }

    var module = module || { exports: {} };
    module.exports = async function main() {
      await some_async_func();
    }
  `)
  console.log('It should be an error')
  console.log(ret)
  console.log(ret.isError)
  console.log(ret.error)
}
