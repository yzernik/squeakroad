function getImageSrcString(imageBase64) {
  return `data:image/jpeg;base64,${imageBase64}`;
}

export function getProfileImageSrcString(squeakProfile) {
  const profileImage = squeakProfile.getProfileImage();
  return getImageSrcString(profileImage);
}
