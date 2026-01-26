/**
 * About page with project information.
 */
export function About() {
  return (
    <div className="max-w-4xl mx-auto">
      <h1 className="text-3xl font-bold text-gray-900 mb-6">
        About Apportionment
      </h1>

      <div className="bg-white shadow rounded-lg p-6 space-y-6">
        <section>
          <h2 className="text-xl font-semibold text-gray-900 mb-3">
            Overview
          </h2>
          <p className="text-gray-700 leading-relaxed">
            This dashboard provides a web interface for running and visualizing
            algorithmic congressional redistricting using METIS recursive bisection.
            The system generates compact, population-balanced districts for all 50 states
            across three census years (2000, 2010, 2020) without considering political
            or racial data.
          </p>
        </section>

        <section>
          <h2 className="text-xl font-semibold text-gray-900 mb-3">
            Algorithm
          </h2>
          <p className="text-gray-700 leading-relaxed mb-2">
            The redistricting algorithm uses:
          </p>
          <ul className="list-disc list-inside text-gray-700 space-y-1 ml-4">
            <li>METIS recursive bisection for graph partitioning</li>
            <li>Edge-weighted graphs for compactness optimization</li>
            <li>Population balancing within ±0.5% tolerance</li>
            <li>Contiguity enforcement (all districts connected)</li>
          </ul>
        </section>

        <section>
          <h2 className="text-xl font-semibold text-gray-900 mb-3">
            Features
          </h2>
          <ul className="list-disc list-inside text-gray-700 space-y-1 ml-4">
            <li>Create and manage redistricting pipeline runs</li>
            <li>Real-time progress monitoring with STATUS protocol</li>
            <li>Multi-year parallel execution (2000, 2010, 2020)</li>
            <li>State-level filtering for testing and analysis</li>
            <li>Configurable worker processes for performance tuning</li>
          </ul>
        </section>

        <section>
          <h2 className="text-xl font-semibold text-gray-900 mb-3">
            Technology Stack
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <h3 className="font-semibold text-gray-900 mb-2">Backend</h3>
              <ul className="list-disc list-inside text-gray-700 space-y-1 ml-4 text-sm">
                <li>FastAPI (Python 3.11+)</li>
                <li>PostgreSQL database</li>
                <li>SQLAlchemy ORM</li>
                <li>Async subprocess management</li>
              </ul>
            </div>
            <div>
              <h3 className="font-semibold text-gray-900 mb-2">Frontend</h3>
              <ul className="list-disc list-inside text-gray-700 space-y-1 ml-4 text-sm">
                <li>React 18 + TypeScript</li>
                <li>Vite build tool</li>
                <li>Tailwind CSS</li>
                <li>React Query for data fetching</li>
              </ul>
            </div>
          </div>
        </section>

        <section className="border-t pt-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-3">
            Documentation
          </h2>
          <div className="flex gap-3">
            <a
              href="/docs"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:text-blue-800 text-sm underline"
            >
              API Documentation
            </a>
            <a
              href="https://github.com/anthropics/apportionment"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:text-blue-800 text-sm underline"
            >
              GitHub Repository
            </a>
          </div>
        </section>
      </div>
    </div>
  )
}
