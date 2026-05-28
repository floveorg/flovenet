export const REGISTER = `
  mutation Register($email: String!, $password: String!, $displayName: String!) {
    register(email: $email, password: $password, displayName: $displayName) {
      token
      profile {
        peerId
        displayName
        bio
        avatarCid
        followerCount
        followingCount
        postCount
      }
    }
  }
`

export const LOGIN = `
  mutation Login($email: String!, $password: String!) {
    login(email: $email, password: $password) {
      token
      profile {
        peerId
        displayName
        bio
        avatarCid
        followerCount
        followingCount
        postCount
      }
    }
  }
`

export const GET_PROFILE = `
  query Profile($peerId: String!) {
    profile(peerId: $peerId) {
      peerId
      displayName
      bio
      avatarCid
      followerCount
      followingCount
      postCount
      reputation
    }
  }
`

export const GET_FEED = `
  query Feed($limit: Int, $offset: Int) {
    feed(limit: $limit, offset: $offset) {
      post {
        cid
        content
        media
        parent
        timestamp
      }
      author {
        peerId
        displayName
      }
      score
    }
  }
`

export const SEARCH_PROFILES = `
  query SearchProfiles($query: String!) {
    searchProfiles(query: $query) {
      peerId
      displayName
      bio
      followerCount
    }
  }
`

export const CREATE_POST = `
  mutation CreatePost($content: String!) {
    createPost(content: $content) {
      cid
      content
      timestamp
    }
  }
`

export const FOLLOW = `
  mutation Follow($peerId: String!) {
    follow(peerId: $peerId)
  }
`

export const UNFOLLOW = `
  mutation Unfollow($peerId: String!) {
    unfollow(peerId: $peerId)
  }
`

export const UPDATE_PROFILE = `
  mutation UpdateProfile($displayName: String, $bio: String, $avatarCid: String) {
    updateProfile(displayName: $displayName, bio: $bio, avatarCid: $avatarCid) {
      peerId
      displayName
      bio
      avatarCid
    }
  }
`

export const GET_FOLLOWING = `
  query Following($peerId: String!) {
    following(peerId: $peerId) {
      peerId
      displayName
    }
  }
`

export const GET_FOLLOWERS = `
  query Followers($peerId: String!) {
    followers(peerId: $peerId) {
      peerId
      displayName
    }
  }
`
