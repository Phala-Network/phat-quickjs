var module = module || { exports: {} };
module.exports = async function main() {
  const ret = await Wapo.run(`
    const bar = {
      foo: __something_not_exists__.bar
    }
  `)
  console.log('It should be an error')
  console.log(ret)
  console.log(ret.isError)
  console.log(ret.error)
}
