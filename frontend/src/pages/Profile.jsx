import React, { useEffect, useState, useContext } from "react";
import { useNavigate, useParams } from "react-router-dom";
import axios from "axios";
import { AuthContext } from "../authcontext.jsx";
import PhoneInput from "react-phone-input-2";
import "react-phone-input-2/lib/style.css";

const API_BASE = "http://localhost:8081/api/user/auth";

function Profile() {
  const { id } = useParams();
  const { user, token } = useContext(AuthContext);
  const navigate = useNavigate();

  const [profile, setProfile] = useState(null);
  const [loadingProfile, setLoadingProfile] = useState(true);
  const [editing, setEditing] = useState(false);
  const [followersCount, setFollowersCount] = useState(0);
  const [followingCount, setFollowingCount] = useState(0);
  const [formData, setFormData] = useState({
    username: "",
    email: "",
    accountType: "public",
    phoneNo: "",
    address: "",
  });

  const [pendingRequests, setPendingRequests] = useState([]);
  const [followersList, setFollowersList] = useState([]);
  const [followersPage, setFollowersPage] = useState(1);
  const [hasMoreFollowers, setHasMoreFollowers] = useState(true);
  const [loadingFollowers, setLoadingFollowers] = useState(false);

  const [followingList, setFollowingList] = useState([]);
  const [followingPage, setFollowingPage] = useState(1);
  const [hasMoreFollowing, setHasMoreFollowing] = useState(true);
  const [loadingFollowing, setLoadingFollowing] = useState(false);

  const itemsPerPage = 6;
  const [showFollowers, setShowFollowers] = useState(false);
  const [showFollowing, setShowFollowing] = useState(false);

  const isOwner = !id || (user && String(user.id) === String(id));

  // ✅ Fetch Profile
  useEffect(() => {
    const fetchProfile = async () => {
      try {
        if (!token) return;

        const profileId = id || user.id;
        const url = `${API_BASE}/profile/${profileId}`;

        const res = await axios.get(url, {
          headers: { Authorization: `Bearer ${token}` },
        });

        const profileData = res.data;
        setProfile(profileData);
        setFollowersCount(profileData.FollowersCount || 0);
        setFollowingCount(profileData.FollowingCount || 0);

        setFormData({
          username: profileData.username || "",
          email: profileData.email || "",
          accountType: profileData.accountType || "public",
          phoneNo: profileData.phoneNo || "",
          address: profileData.address || "",
        });

        // Load pending requests only if owner + private account
        if (isOwner && profileData.accountType?.toLowerCase() === "private") {
          const requestsRes = await axios.get(
            `${API_BASE}/follow-req/${profileData.id}`,
            { headers: { Authorization: `Bearer ${token}` } }
          );
          setPendingRequests(requestsRes.data.pendingRequests || []);
        }
      } catch (err) {
        console.error("Error fetching profile:", err);
      } finally {
        setLoadingProfile(false);
      }
    };

    fetchProfile();
  }, [id, token, isOwner, user?.id]);

  // ✅ Followers
  const fetchFollowers = async (page = 1) => {
    if (!profile?.id) return;
    setLoadingFollowers(true);
    try {
      const res = await axios.get(
        `${API_BASE}/followers/${profile.id}?page=${page}&limit=${itemsPerPage}`,
        { headers: { Authorization: `Bearer ${token}` } }
      );

      const data = res.data.followers || [];
      setFollowersList((prev) => (page === 1 ? data : [...prev, ...data]));
      setHasMoreFollowers(data.length === itemsPerPage);
      setFollowersPage(page);
    } catch (err) {
      console.error("Error fetching followers:", err);
    } finally {
      setLoadingFollowers(false);
    }
  };

  // ✅ Following
  const fetchFollowing = async (page = 1) => {
    if (!profile?.id) return;
    setLoadingFollowing(true);
    try {
      const res = await axios.get(
        `${API_BASE}/followings/${profile.id}?page=${page}&limit=${itemsPerPage}`,
        { headers: { Authorization: `Bearer ${token}` } }
      );
      console.log("Following data:", res.data);
      const data = res.data.following || [];
      setFollowingList((prev) => (page === 1 ? data : [...prev, ...data]));
      setHasMoreFollowing(data.length === itemsPerPage);
      setFollowingPage(page);
    } catch (err) {
      console.error("Error fetching following:", err);
    } finally {
      setLoadingFollowing(false);
    }
  };

  // ✅ Handle changes
  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  // ✅ Update profile
  const handleUpdate = async () => {
    try {
      const formattedPhone =
        formData.phoneNo && !formData.phoneNo.startsWith("+")
          ? `+${formData.phoneNo}`
          : formData.phoneNo;

      const url = `${API_BASE}/profile-update/${id || user.id}`;
      const res = await axios.put(
        url,
        {
          username: formData.username,
          email: formData.email,
          accountType: formData.accountType,
          phoneNo: formattedPhone,
          address: formData.address,
          loggedInUserId: user.id,
        },
        { headers: { Authorization: `Bearer ${token}` } }
      );

      const updatedProfile = res.data;
      setProfile(updatedProfile);
      setEditing(false);
      alert("Profile updated successfully!");
    } catch (err) {
      console.error("Failed to update profile:", err);
      alert("Failed to update profile");
    }
  };

  // ✅ Handle follow request actions
  const handleRequestAction = async (requestId, action) => {
    try {
      await axios.post(
        `${API_BASE}/handle-follow-req/${requestId}`,
        { ownerId: user.id, action },
        { headers: { Authorization: `Bearer ${token}` } }
      );
      setPendingRequests((prev) => prev.filter((r) => r.id !== requestId));
      
      if (action === "approve") {
        setFollowersCount((prev) => prev + 1);
      }
      
      alert(`Request ${action}ed successfully`);
    } catch (err) {
      console.error("Follow request action failed:", err);
      alert(`Failed to ${action} request`);
    }
  };

  // ✅ Follow/Unfollow - FIXED
  const handleFollowAction = async (targetUserId, action) => {
    try {
      await axios.post(
        "http://localhost:8081/api/user/auth/follow",
        { 
          userId: user.id, 
          targetId: targetUserId,
          action, 
          isRequest: false
        },
        { headers: { Authorization: `Bearer ${token}` } }
      );

      if (action === "unfollow") {
        setFollowingCount((prev) => Math.max(prev - 1, 0));
        setFollowingList((prev) => prev.filter((f) => f.id !== targetUserId));
      } else if (action === "follow") {
        setFollowingCount((prev) => prev + 1);
      }
      
      alert(`Successfully ${action}ed the user.`);
    } catch (err) {
      console.error(`Failed to ${action}:`, err);
      alert(`Failed to ${action} user`);
    }
  };

  // ✅ UI Rendering
  if (loadingProfile)
    return (
      <div className="d-flex justify-content-center align-items-center vh-100">
        <div className="text-center">
          <div className="spinner-border text-primary" role="status" style={{ width: "3rem", height: "3rem" }}>
            <span className="visually-hidden">Loading...</span>
          </div>
          <p className="mt-3 text-muted">Loading profile...</p>
        </div>
      </div>
    );

  if (!profile)
    return (
      <div className="container mt-5">
        <div className="alert alert-warning text-center" role="alert">
          <i className="bi bi-exclamation-triangle-fill me-2"></i>
          Profile not found
        </div>
      </div>
    );

  return (
    <div className="container-fluid py-4" style={{ backgroundColor: "#f8f9fa", minHeight: "100vh" }}>
      <div className="container">
        <button
          className="btn btn-outline-primary mb-4 shadow-sm"
          onClick={() => navigate("/home")}
        >
          <i className="bi bi-arrow-left me-2"></i>Back to Home
        </button>

        <div className="row g-4">
          {/* LEFT SIDE */}
          <div className="col-lg-8">
            {/* Profile Card */}
            <div className="card border-0 shadow-sm mb-4">
              <div className="card-body p-4">
                <div className="d-flex align-items-start mb-4">
                  <div className="position-relative">
                    <img
                      src={
                        profile.profile_pic
                          ? `http://127.0.0.1:8081/profile_pic/${profile.profile_pic}`
                          : "/default-profile.png"
                      }
                      alt={profile.username}
                      className="rounded-circle border border-3 border-primary shadow"
                      style={{
                        width: "120px",
                        height: "120px",
                        objectFit: "cover",
                      }}
                    />
                    {profile.accountType?.toLowerCase() === "private" && (
                      <span className="position-absolute bottom-0 end-0 badge bg-warning text-dark rounded-circle p-2">
                        <i className="bi bi-lock-fill"></i>
                      </span>
                    )}
                  </div>
                  
                  <div className="ms-4 flex-grow-1">
                    <div className="d-flex align-items-center justify-content-between mb-3">
                      <h3 className="mb-0 fw-bold">{profile.username}</h3>
                      {isOwner && (
                        <button
                          className="btn btn-primary btn-sm shadow-sm"
                          onClick={() => setEditing(true)}
                        >
                          <i className="bi bi-pencil-square me-1"></i>Edit Profile
                        </button>
                      )}
                    </div>

                    <div className="d-flex gap-4 mb-3">
                      <div
                        className="text-center cursor-pointer"
                        style={{ cursor: "pointer" }}
                        onClick={() => {
                          fetchFollowers(1);
                          setShowFollowers(true);
                          setShowFollowing(false);
                        }}
                      >
                        <div className="fs-4 fw-bold text-primary">{followersCount}</div>
                        <small className="text-muted">Followers</small>
                      </div>
                      <div
                        className="text-center cursor-pointer"
                        style={{ cursor: "pointer" }}
                        onClick={() => {
                          fetchFollowing(1);
                          setShowFollowing(true);
                          setShowFollowers(false);
                        }}
                      >
                        <div className="fs-4 fw-bold text-success">{followingCount}</div>
                        <small className="text-muted">Following</small>
                      </div>
                    </div>

                    <span className={`badge ${profile.accountType?.toLowerCase() === "private" ? "bg-warning text-dark" : "bg-success"} shadow-sm`}>
                      <i className={`bi ${profile.accountType?.toLowerCase() === "private" ? "bi-lock-fill" : "bi-globe"} me-1`}></i>
                      {profile.accountType}
                    </span>
                  </div>
                </div>

                <hr className="my-4" />

                {/* Edit Mode */}
                {editing ? (
                  <div className="animate__animated animate__fadeIn">
                    <h5 className="mb-3 text-primary">
                      <i className="bi bi-pencil-square me-2"></i>Edit Profile
                    </h5>
                    <div className="row g-3">
                      <div className="col-md-6">
                        <label className="form-label fw-semibold">
                          <i className="bi bi-person me-1"></i>Username
                        </label>
                        <input
                          className="form-control shadow-sm"
                          name="username"
                          value={formData.username}
                          onChange={handleChange}
                          placeholder="Enter username"
                        />
                      </div>
                      <div className="col-md-6">
                        <label className="form-label fw-semibold">
                          <i className="bi bi-envelope me-1"></i>Email
                        </label>
                        <input
                          className="form-control shadow-sm"
                          name="email"
                          type="email"
                          value={formData.email}
                          onChange={handleChange}
                          placeholder="Enter email"
                        />
                      </div>
                      <div className="col-md-6">
                        <label className="form-label fw-semibold">
                          <i className="bi bi-telephone me-1"></i>Phone Number
                        </label>
                        <PhoneInput
                          country={"us"}
                          value={formData.phoneNo}
                          onChange={(phone) =>
                            setFormData((prev) => ({ ...prev, phoneNo: phone }))
                          }
                          inputProps={{
                            name: "phoneNo",
                            required: true,
                            className: "form-control shadow-sm",
                          }}
                          containerClass="w-100"
                        />
                      </div>
                      <div className="col-md-6">
                        <label className="form-label fw-semibold">
                          <i className="bi bi-shield-lock me-1"></i>Account Type
                        </label>
                        <select
                          className="form-select shadow-sm"
                          name="accountType"
                          value={formData.accountType}
                          onChange={handleChange}
                        >
                          <option value="public">🌍 Public</option>
                          <option value="private">🔒 Private</option>
                        </select>
                      </div>
                      <div className="col-12">
                        <label className="form-label fw-semibold">
                          <i className="bi bi-geo-alt me-1"></i>Address
                        </label>
                        <input
                          className="form-control shadow-sm"
                          name="address"
                          value={formData.address}
                          onChange={handleChange}
                          placeholder="Enter address"
                        />
                      </div>
                    </div>
                    <div className="d-flex gap-2 mt-4">
                      <button className="btn btn-success shadow-sm px-4" onClick={handleUpdate}>
                        <i className="bi bi-check-circle me-1"></i>Save Changes
                      </button>
                      <button
                        className="btn btn-secondary shadow-sm px-4"
                        onClick={() => setEditing(false)}
                      >
                        <i className="bi bi-x-circle me-1"></i>Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className="row g-3">
                    <div className="col-md-6">
                      <div className="p-3 bg-light rounded">
                        <small className="text-muted d-block mb-1">
                          <i className="bi bi-envelope me-1"></i>Email
                        </small>
                        <div className="fw-semibold">
                          {isOwner || profile.accountType?.toLowerCase() === "public"
                            ? profile.email
                            : <span className="text-muted"><i className="bi bi-lock-fill me-1"></i>Private</span>}
                        </div>
                      </div>
                    </div>
                    <div className="col-md-6">
                      <div className="p-3 bg-light rounded">
                        <small className="text-muted d-block mb-1">
                          <i className="bi bi-telephone me-1"></i>Phone Number
                        </small>
                        <div className="fw-semibold">
                          {isOwner || profile.accountType?.toLowerCase() === "public"
                            ? profile.phoneNo || "-"
                            : <span className="text-muted"><i className="bi bi-lock-fill me-1"></i>Private</span>}
                        </div>
                      </div>
                    </div>
                    <div className="col-12">
                      <div className="p-3 bg-light rounded">
                        <small className="text-muted d-block mb-1">
                          <i className="bi bi-geo-alt me-1"></i>Address
                        </small>
                        <div className="fw-semibold">
                          {isOwner || profile.accountType?.toLowerCase() === "public"
                            ? profile.address || "-"
                            : <span className="text-muted"><i className="bi bi-lock-fill me-1"></i>Private</span>}
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Followers List */}
            {showFollowers && (
              <div className="card border-0 shadow-sm mb-4">
                <div className="card-header bg-primary text-white">
                  <h5 className="mb-0">
                    <i className="bi bi-people-fill me-2"></i>
                    Followers ({followersList.length})
                  </h5>
                </div>
                <div className="card-body">
                  {loadingFollowers ? (
                    <div className="text-center py-4">
                      <div className="spinner-border text-primary" role="status">
                        <span className="visually-hidden">Loading...</span>
                      </div>
                    </div>
                  ) : followersList.length === 0 ? (
                    <div className="text-center py-4 text-muted">
                      <i className="bi bi-person-x fs-1 d-block mb-2"></i>
                      No followers found
                    </div>
                  ) : (
                    <>
                      <div className="row g-3">
                        {followersList.map((f, index) => (
                          <div key={f.id || `follower-${index}`} className="col-12">
                            <div className="d-flex align-items-center justify-content-between p-3 bg-light rounded shadow-sm">
                              <div className="d-flex align-items-center">
                                <img
                                  src={
                                    f.profile_pic
                                      ? `http://127.0.0.1:8081/profile_pic/${f.profile_pic}`
                                      : "/default-profile.png"
                                  }
                                  alt={f.username}
                                  className="rounded-circle border border-2 border-primary me-3"
                                  style={{
                                    width: "50px",
                                    height: "50px",
                                    objectFit: "cover",
                                  }}
                                />
                                <span className="fw-semibold">{f.name}</span>
                              </div>
                              {!isOwner && (
                                <button
                                  className="btn btn-primary btn-sm shadow-sm"
                                  onClick={() => handleFollowAction(f.id, "follow")}
                                >
                                  <i className="bi bi-person-plus me-1"></i>Follow
                                </button>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                      {hasMoreFollowers && (
                        <button
                          className="btn btn-outline-primary w-100 mt-3"
                          onClick={() => fetchFollowers(followersPage + 1)}
                          disabled={loadingFollowers}
                        >
                          <i className="bi bi-arrow-down-circle me-1"></i>Load More
                        </button>
                      )}
                    </>
                  )}
                </div>
              </div>
            )}

            {/* Following List */}
            {showFollowing && (
              <div className="card border-0 shadow-sm mb-4">
                <div className="card-header bg-success text-white">
                  <h5 className="mb-0">
                    <i className="bi bi-person-check-fill me-2"></i>
                    Following ({followingList.length})
                  </h5>
                </div>
                <div className="card-body">
                  {loadingFollowing ? (
                    <div className="text-center py-4">
                      <div className="spinner-border text-success" role="status">
                        <span className="visually-hidden">Loading...</span>
                      </div>
                    </div>
                  ) : followingList.length === 0 ? (
                    <div className="text-center py-4 text-muted">
                      <i className="bi bi-person-x fs-1 d-block mb-2"></i>
                      Not following anyone yet
                    </div>
                  ) : (
                    <>
                      <div className="row g-3">
                        {followingList.map((f, index) => (
                          <div key={f.id || `following-${index}`} className="col-12">
                            <div className="d-flex align-items-center justify-content-between p-3 bg-light rounded shadow-sm">
                              <div className="d-flex align-items-center">
                                <img
                                  src={
                                    f.profile_pic
                                      ? `http://127.0.0.1:8081/profile_pic/${f.profile_pic}`
                                      : "/default-profile.png"
                                  }
                                  alt={f.username}
                                  className="rounded-circle border border-2 border-success me-3"
                                  style={{
                                    width: "50px",
                                    height: "50px",
                                    objectFit: "cover",
                                  }}
                                />
                                <span className="fw-semibold">{f.name}</span>
                              </div>
                              {isOwner && (
                                <button
                                  className="btn btn-outline-danger btn-sm shadow-sm"
                                  onClick={() => handleFollowAction(f.id, "unfollow")}
                                >
                                  <i className="bi bi-person-dash me-1"></i>Unfollow
                                </button>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                      {hasMoreFollowing && (
                        <button
                          className="btn btn-outline-success w-100 mt-3"
                          onClick={() => fetchFollowing(followingPage + 1)}
                          disabled={loadingFollowing}
                        >
                          <i className="bi bi-arrow-down-circle me-1"></i>Load More
                        </button>
                      )}
                    </>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* RIGHT SIDE - Pending Requests */}
          {isOwner && profile.accountType?.toLowerCase() === "private" && (
            <div className="col-lg-4">
              <div className="card border-0 shadow-sm sticky-top" style={{ top: "20px" }}>
                <div className="card-header bg-warning text-dark">
                  <h5 className="mb-0">
                    <i className="bi bi-bell-fill me-2"></i>
                    Pending Requests ({pendingRequests.length})
                  </h5>
                </div>
                <div className="card-body" style={{ maxHeight: "600px", overflowY: "auto" }}>
                  {pendingRequests.length === 0 ? (
                    <div className="text-center py-4 text-muted">
                      <i className="bi bi-inbox fs-1 d-block mb-2"></i>
                      No pending requests
                    </div>
                  ) : (
                    <div className="d-flex flex-column gap-3">
                      {pendingRequests.map((req, index) => (
                        <div
                          key={req.id || `request-${index}`}
                          className="p-3 bg-light rounded shadow-sm"
                        >
                          <div className="d-flex align-items-center mb-3">
                            <img
                              src={
                                req.profile_pic
                                  ? `http://127.0.0.1:8081/profile_pic/${req.profile_pic}`
                                  : "/default-profile.png"
                              }
                              alt={req.username}
                              className="rounded-circle border border-2 border-warning me-3"
                              style={{
                                width: "50px",
                                height: "50px",
                                objectFit: "cover",
                              }}
                            />
                            <span className="fw-semibold">{req.username}</span>
                          </div>
                          <div className="d-flex gap-2">
                            <button
                              className="btn btn-success btn-sm flex-fill shadow-sm"
                              onClick={() => handleRequestAction(req.id, "approve")}
                            >
                              <i className="bi bi-check-circle me-1"></i>Accept
                            </button>
                            <button
                              className="btn btn-danger btn-sm flex-fill shadow-sm"
                              onClick={() => handleRequestAction(req.id, "reject")}
                            >
                              <i className="bi bi-x-circle me-1"></i>Decline
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default Profile;