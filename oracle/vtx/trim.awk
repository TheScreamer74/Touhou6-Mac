# Keep the anm interpreter + loader + Draw3 (the world-transform build we want to
# verify); delete the other D3D draw/surface/texture methods (stubbed separately).
# Same as oracle/anm/trim.awk but Draw3 is NOT in the skip list.
BEGIN { skip = 0 }
{
  if (skip) { if ($0 == "}") skip = 0; next }
  if ($0 ~ /AnmManager::(Draw|Draw2|DrawEndingRect|DrawFacingCamera|DrawInner|DrawNoRotation|DrawStringFormat|DrawStringFormat2|DrawTextToSprite|DrawVmTextFmt|CopySurfaceToBackBuffer|SetRenderStateForVm|SetupVertexBuffer|TakeScreenshot|TakeScreenshotIfRequested|TranslateRotation|LoadSurface|ReleaseSurface|ReleaseSurfaces|LoadTexture|CreateEmptyTexture|LoadTextureAlphaChannel|ReleaseTexture)\(/) {
    skip = 1; next
  }
  print
}
