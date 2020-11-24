import { NextPage } from 'next'
import Nav from '../components/nav'

const IndexPage: NextPage = () => {
  return (
    <main>
      <Nav />
      <div className="py-20">
        <h1 className="text-5xl text-center text-gray-700 dark:text-gray-100">emmy.leaf.codes</h1>
      </div>
    </main>
  )
}

export default IndexPage
