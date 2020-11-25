import { NextComponentType } from 'next'

const leftLinks = [
  { href: 'https://emmy.leaf.codes', label: 'emmy.leaf.codes' },
  // { href: 'https://music.leaf.codes', label: 'music.leaf.codes' },
]

const rightLinks = [
  { href: 'https://github.com/emmyleaf', label: 'GitHub' },
  // { href: 'https://nextjs.org/docs', label: 'Docs' },
]

const Nav: NextComponentType = () => {
  return (
    <nav className="flex justify-between p-8">
      <ul className="flex flex-col items-left justify-between space-y-4">
        {leftLinks.map(({ href, label }) => (
          <li key={`${href}${label}`}>
            <a href={href} className="no-underline button">
              {label}
            </a>
          </li>
        ))}
      </ul>
      <ul className="flex items-right justify-between space-x-4">
        {rightLinks.map(({ href, label }) => (
          <li key={`${href}${label}`}>
            <a href={href} className="no-underline button">
              {label}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  )
}

export default Nav
