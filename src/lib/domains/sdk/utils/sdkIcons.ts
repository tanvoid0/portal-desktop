export function getSDKIcon(sdkType: string): string {
  const iconMap: Record<string, string> = {
    // Language & Runtime
    java: 'devicon-java-plain',
    node: 'devicon-nodejs-plain',
    python: 'devicon-python-plain',
    rust: 'devicon-rust-plain',
    go: 'devicon-go-plain',
    php: 'devicon-php-plain',
    ruby: 'devicon-ruby-plain',
    bun: 'devicon-bun-plain',
    deno: 'devicon-deno-plain',
    gradle: 'devicon-gradle-plain',
    kotlin: 'devicon-kotlin-plain',
    scala: 'devicon-scala-plain',
    erlang: 'devicon-erlang-plain',
    perl: 'devicon-perl-plain',

    // Database
    mysql: 'devicon-mysql-plain',
    postgresql: 'devicon-postgresql-plain',
    mongodb: 'devicon-mongodb-plain',
    mariadb: 'devicon-mariadb-plain',

    // Web Server
    nginx: 'devicon-nginx-original',
    apache: 'devicon-apache-plain',
    caddy: 'devicon-caddy-plain',

    // Container
    docker: 'devicon-docker-plain',
    kubernetes: 'devicon-kubernetes-plain',
    podman: 'devicon-podman-plain',

    // Package Managers
    npm: 'devicon-npm-original-wordmark',
    yarn: 'devicon-yarn-plain',
    pip: 'devicon-python-plain',
    cargo: 'devicon-rust-plain',
    composer: 'devicon-composer-plain',
    gem: 'devicon-ruby-plain',

    // SDK Managers
    nvm: 'devicon-nodejs-plain',
    pyenv: 'devicon-python-plain',
    rustup: 'devicon-rust-plain',
    sdkman: 'devicon-sdkman-plain',
    goenv: 'devicon-go-plain',
    rbenv: 'devicon-ruby-plain',
    phpenv: 'devicon-php-plain',
    fnm: 'devicon-nodejs-plain',
  };

  return iconMap[sdkType.toLowerCase()] || 'devicon-devicon-plain';
}

export function getSDKIconColor(sdkId: string): string {
  const colorMap: Record<string, string> = {
    // Language & Runtime - colored icons
    java: 'text-orange-600',
    node: 'text-green-600',
    nodejs: 'text-green-600',
    python: 'text-blue-600',
    rust: 'text-orange-600',
    go: 'text-blue-500',
    php: 'text-purple-600',
    ruby: 'text-red-600',
    bun: 'text-yellow-600',
    deno: 'text-gray-800',
    gradle: 'text-blue-500',
    kotlin: 'text-purple-600',
    scala: 'text-red-600',
    erlang: 'text-red-500',
    perl: 'text-blue-700',

    // Database
    mysql: 'text-blue-600',
    postgresql: 'text-blue-700',
    mongodb: 'text-green-600',
    mariadb: 'text-blue-500',

    // Web Server
    nginx: 'text-green-600',
    apache: 'text-red-600',
    caddy: 'text-blue-600',

    // Container
    docker: 'text-blue-500',
    kubernetes: 'text-blue-600',
    podman: 'text-blue-600',

    // Package Managers
    npm: 'text-red-600',
    yarn: 'text-blue-600',
    pip: 'text-blue-600',
    cargo: 'text-orange-600',
    composer: 'text-gray-700',
    gem: 'text-red-600',

    // SDK Managers
    nvm: 'text-green-600',
    pyenv: 'text-blue-600',
    rustup: 'text-orange-600',
    sdkman: 'text-blue-600',
    goenv: 'text-blue-500',
    rbenv: 'text-red-600',
    phpenv: 'text-purple-600',
  };

  return colorMap[sdkId.toLowerCase()] || '';
}
